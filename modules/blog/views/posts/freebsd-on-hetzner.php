+++
path = "blog/freebsd-on-hetzner"
title = "FreeBSD on a Hetzner Cloud VPS"
date = 2026-07-26
description = "Hetzner has no FreeBSD image, and the installer everyone recommends never verifies what it downloads before handing it root. In this article I install 15.1 on root-on-ZFS the honest way, and walk through the five things that broke on the way there."
hero = "/images/freebsd-on-hetzner/header.svg"
hero_alt = "FreeBSD 15.1 on root-on-ZFS, with the disk partition layout"
card = "/images/freebsd-on-hetzner/card.png"
card_alt = "Bar chart of mean latency for a fresh TLS connection at 100, 250 and 500 connections a second: FreeBSD 4.9, 4.7 and 4.0 milliseconds, Ubuntu 5.7, 5.0 and 5.2"
+++
<p>Hetzner has no FreeBSD image and their ISO library stops at 15.0. You can still get <strong>any</strong> release on there, unattended, in about an hour.</p>
<blockquote>
<p>The box runs AthletOS, a training app I am building. Predefined programs, session logging and tracking.</p>
<p>— I picked FreeBSD for ZFS boot environments and jails, so a bad deploy is a rollback instead of an incident.</p>
</blockquote>
<p>This is the whole procedure, in order, with the things that went wrong and what fixed them. I used FreeBSD’s own ISO rather than a prebuilt image, so the checksum can be verified against the project’s manifest.</p>
<?= section_open('1. Create the server') ?>
<p>A CX23 gives 2 vCPU and 4 GB, which is plenty. Firewall first, then the box.</p>
<?= code_open('bash', '1-create-server.sh') ?>
hcloud firewall create --name athletos
for p in 22 80 443; do
  hcloud firewall add-rule athletos --direction in --protocol tcp \
    --port $p --source-ips 0.0.0.0/0 --source-ips ::/0
done

hcloud server create --name athletos --type cx23 --location hel1 \
  --image debian-12 --ssh-key my-key --firewall athletos
<?= code_close() ?>
<p>The image is a throwaway. It only exists so the server boots at all.</p>
<?= section_close() ?>

<?= section_open('2. Boot into rescue') ?>
<?= code_open('bash', '2-rescue.sh') ?>
hcloud server enable-rescue athletos --ssh-key my-key
hcloud server reset athletos

ssh root@<ip>          # linux rescue, ~1.9GB overlay in RAM
<?= code_close() ?>
<?= section_close() ?>

<?= section_open('3. Fetch and verify the ISO') ?>
<p>Take <code>bootonly</code>, not <code>disc1</code>. The rescue root is a small RAM overlay and a 1.3 GB ISO plus its extraction plus the rebuild will not fit.</p>
<?= code_open('bash', '3-fetch-iso.sh') ?>
B=https://download.freebsd.org/releases/amd64/amd64/ISO-IMAGES/15.1
I=FreeBSD-15.1-RELEASE-amd64-bootonly.iso

curl -fsSL -o CHECKSUM $B/CHECKSUM.SHA256-FreeBSD-15.1-RELEASE-amd64
curl -fsSL -o $I $B/$I

E=$(grep -E "\($I\)" CHECKSUM | sed 's/.*= //')
A=$(sha256sum $I | cut -d' ' -f1)
[ "$E" = "$A" ] && echo "verified" || echo "STOP"
<?= code_close() ?>
<?= section_close() ?>

<?= section_open('4. Remaster the ISO') ?>
<p>Three edits: a serial console so we can watch it, the prompt that would otherwise block forever, and the network the scripted install never configures for itself.</p>
<?= code_open('bash', '4-remaster.sh') ?>
apt-get install -y xorriso qemu-system-x86

mkdir -p isowork isomnt
mount -o loop,ro $I isomnt && cp -a isomnt/. isowork/ && umount isomnt
chmod -R u+w isowork

# append, never replace: the original carries vfs.root.mountfrom
cat >> isowork/boot/loader.conf <<'EOF'
console="comconsole"
comconsole_speed="115200"
boot_serial="YES"
vfs.root.mountfrom="cd9660:/dev/iso9660/FREEBSD_INSTALL"
EOF

# the serial path asks for a terminal type with an unconditional read
sed -i 's/^\([[:space:]]*\)read TERM$/\1:/' \
    isowork/usr/libexec/bsdinstall/startbsdinstall

# a scripted install never configures the network
sed -i 's|^mkdir /tmp/bsdinstall_etc$|&\nifconfig vtnet0 inet 10.0.2.15/24 up\nroute add default 10.0.2.2\necho "nameserver 10.0.2.3" > /tmp/bsdinstall_etc/resolv.conf|' \
    isowork/etc/rc.local
<?= code_close() ?>
<?= callout_open('warning', 'What went wrong') ?>
<p>I replaced <code>loader.conf</code> instead of appending, which threw away <code>vfs.root.mountfrom</code> and left it at a mountroot prompt.</p>
<p>Then it hung at <code>Console type [vt100]:</code> because that <code>read</code> runs before the script is ever looked for.</p>
<p>Then distfetch died on DNS, because the interactive path exports <code>BSDINSTALL_CONFIGCURRENT</code> and the scripted path does not.</p>
<?= callout_close() ?>
<?= section_close() ?>

<?= section_open('5. Write the unattended install script') ?>
<?= code_open('bash', 'isowork/etc/installerconfig') ?>
PARTITIONS=DEFAULT
DISTRIBUTIONS="kernel.txz base.txz"
export ZFSBOOT_VDEV_TYPE=stripe
export ZFSBOOT_DISKS=vtbd0
export ZFSBOOT_SWAP_SIZE=2g
export ZFSBOOT_CONFIRM_LAYOUT=0
export nonInteractive=YES
export BSDINSTALL_DISTSITE=http://10.0.2.2:8000

#!/bin/sh
sysrc hostname="athletos"
sysrc ifconfig_vtnet0="DHCP"
sysrc sshd_enable="YES"
sysrc zfs_enable="YES"

# ARC takes half of RAM by default. On a 4GB box that is 2GB it does not
# need, and it turns up later looking like a database problem.
echo 'vfs.zfs.arc_max="512M"' >> /boot/loader.conf

mkdir -p /root/.ssh && chmod 700 /root/.ssh
echo 'ssh-ed25519 AAAA... you@host' > /root/.ssh/authorized_keys
chmod 600 /root/.ssh/authorized_keys
<?= code_close() ?>
<p><code>10.0.2.2</code> is QEMU’s host address. Serving the distribution sets from rescue skips DNS entirely and is quicker than fetching them twice.</p>
<?= code_open('bash', '5-serve-dist.sh') ?>
mkdir -p dist && cd dist
B=https://download.freebsd.org/releases/amd64/15.1-RELEASE
for f in MANIFEST base.txz kernel.txz; do curl -fsSL -O $B/$f; done

while read -r name sha rest; do
  [ -f "$name" ] || continue
  [ "$(sha256sum "$name" | cut -d' ' -f1)" = "$sha" ] && echo "OK $name"
done < MANIFEST

setsid nohup python3 -m http.server 8000 >/dev/null 2>&1 &
<?= code_close() ?>
<p>Tip: leave that server running. The installer fetches from it in step 7, not now.</p>
<?= section_close() ?>

<?= section_open('6. Build the ISO and wipe the disk') ?>
<p>Delete the source ISO before building, or you run out of space in the overlay.</p>
<?= code_open('bash', '6-build-and-wipe.sh') ?>
rm -f $I
xorriso -as mkisofs -r -V FREEBSD_INSTALL -b boot/cdboot -no-emul-boot \
        -o athletos-install.iso isowork/
rm -rf isowork

# zfs labels sit at both ends of the device, so wipefs alone is not enough
SECTORS=$(blockdev --getsz /dev/sda)
MIB=$(( SECTORS / 2048 ))
wipefs -a /dev/sda
dd if=/dev/zero of=/dev/sda bs=1M count=64 conv=fsync
dd if=/dev/zero of=/dev/sda bs=1M seek=$(( MIB - 64 )) count=64 conv=fsync
<?= code_close() ?>
<?= callout_open('warning', 'What went wrong') ?>
<p>An earlier attempt got far enough to create a <code>zroot</code> pool before failing.</p>
<p>The next run found it and stopped to ask for a different pool name, which is another dialog nobody is there to answer. Wipe both ends between attempts.</p>
<?= callout_close() ?>
<?= section_close() ?>

<?= section_open('7. Run the installer under QEMU') ?>
<p>Hetzner Cloud has no nested virtualisation, so there is no <code>/dev/kvm</code> and this runs under TCG emulation. It takes roughly half an hour. Nothing to do but watch the log.</p>
<?= code_open('bash', '7-install.sh') ?>
qemu-system-x86_64 -accel tcg,thread=multi -m 2048 -smp 2 \
  -drive file=/dev/sda,format=raw,if=virtio,cache=writeback \
  -cdrom athletos-install.iso -boot d \
  -netdev user,id=n0 -device virtio-net-pci,netdev=n0 \
  -display none -monitor none -serial file:/root/install.log -daemonize

grep -aoE "Running installation step: [a-z]+" /root/install.log | uniq
# zfsboot mount distfetch checksum distextract bootconfig config
# entropy umount script
<?= code_close() ?>
<p>Kill QEMU once the script step finishes, or it reboots straight back into the installer.</p>
<?= section_close() ?>

<?= section_open('8. Boot it, and fix fstab') ?>
<?= code_open('bash', '8-boot.sh') ?>
hcloud server disable-rescue athletos
hcloud server reset athletos

ssh root@<ip>

# the installer ran under qemu virtio (vtbd0), hetzner presents da0.
# the pool is fine because zfs scans devices. swap is not.
printf '/dev/gpt/swap0\tnone\tswap\tsw\t0\t0\n' > /etc/fstab
swapon -a
<?= code_close() ?>
<?= callout_open('warning', 'What went wrong') ?>
<p>The generated fstab pointed at <code>/dev/vtbd0p2</code>, which does not exist on the real machine.</p>
<p>It booted with no swap and said nothing about it. Refer to it by GPT label and the device name stops mattering.</p>
<?= callout_close() ?>
<?= section_close() ?>

<?= section_open('Result') ?>
<?= code_open('bash', 'athletos') ?>
FreeBSD athletos 15.1-RELEASE ... GENERIC amd64

zroot            658M / 34.2G      all pools are healthy
/dev/gpt/swap0   2.0G                     0% used
vfs.zfs.arc_max  536870912                (512 MiB)

1  gptboot0   512 KiB   A501  freebsd-boot
2  swap0      2.0 GiB   A502  freebsd-swap
3  zfs0      36.1 GiB   A504  freebsd-zfs
<?= code_close() ?>
<?= callout_open('note', 'Result') ?>
<p>15.1 on root-on-ZFS, a release the ISO library does not carry, from media you can verify.</p>
<?= callout_close() ?>
<?= section_close() ?>

<?= section_open('Under load') ?>
<p>I measured this from a separate Hetzner VPS in the same datacentre, hel1, against a control: an identical CX23 running Ubuntu 24.04, the same Caddy 2.11.4 serving byte-identical files, with an ECDSA P-256 certificate to match. <code>vegeta</code>, a fresh TCP and TLS connection per request, which is what a crawler does and what a browser never does.</p>
<?= code_open('bash', '9-load.sh') ?>
vegeta attack -targets targets.txt -rate 500 -duration 10s \
       -keepalive=false -timeout 20s | vegeta report

# freebsd   100/s  4.9ms   250/s  4.7ms   500/s  4.0ms   mean
# ubuntu    100/s  5.7ms   250/s  5.0ms   500/s  5.2ms   mean
<?= code_close() ?>
<p>At 100, 250 and 500 fresh connections a second both boxes delivered the full offered rate with 100% success, and the FreeBSD box was the faster of the two at every step.</p>
<p>The crypto is not the constraint either.</p>
<?= code_open('bash', 'openssl speed ecdsap256') ?>
# sign/s
#  37920   this box (epyc, one core)
#  32614   ubuntu control (xeon, one core)
<?= code_close() ?>
<p>The EPYC signs 16% faster than the control’s Xeon, which is worth stating because it means the hardware favours the FreeBSD box rather than the other way round. Against 500 connections a second, 37,920 signatures is roughly 75x headroom. Whatever the ceiling is, it is not asymmetric crypto.</p>
<?= callout_open('warning', 'Two things I did not establish') ?>
<p>Above 500/s the measurements were not reproducible. One clean run at 1000/s had FreeBSD at 497/s and the Ubuntu control at 999/s. A later run gave 179/s at 1000/s, and then 566/s at 1500/s — more offered load producing more throughput, which is not a property a server can have. It is the single load generator accumulating TIME_WAIT between runs. A real answer needs several generators.</p>
<p>An earlier attempt measured this from a Windows laptop over the public internet and got 50-100 connections a second. That was low by five to ten times, and the cause was ephemeral-port exhaustion on the client. The first measurement was wrong, and it was wrong in a way that produced a confident and completely incorrect explanation.</p>
<?= callout_close() ?>
<p>One hypothesis died on the way. FreeBSD’s <code>kern.ipc.soacceptqueue</code> is 128 and Ubuntu’s <code>net.core.somaxconn</code> is 4096, so the listen backlog differs by 32x, which looked like the obvious cause and survived the arithmetic. I set the control to 128, restarted Caddy so the listener picked the new value up, and it still delivered 999.9/s. Refuted.</p>
<p>That leaves <code>pf</code>, which redirects 443 to Caddy’s high port, as the untested suspect. Untested because that high port is not reachable from outside, so there was no way to bypass the redirect and compare against it.</p>
<p>The same steps work for any release. Change the version in step 3 and the rest follows.</p>
<p>Every command above ran against a real CX23 in hel1.</p>
<?= section_close() ?>
