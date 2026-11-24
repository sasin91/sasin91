import { PDFDocument, StandardFonts, rgb, type PDFFont, type PDFPage } from 'pdf-lib'
import { formatCvMonth, getCv, type LocaleCode } from '#shared/data/cv'

const PAGE_WIDTH = 595.28
const PAGE_HEIGHT = 841.89
const MARGIN_X = 46
const MARGIN_TOP = 50
const MARGIN_BOTTOM = 50
const CONTENT_WIDTH = PAGE_WIDTH - MARGIN_X * 2

const PDF_LABELS: Record<
  LocaleCode,
  {
    online: string
    experience: string
    skills: string
    education: string
    present: string
  }
> = {
  en: {
    online: 'Full CV online:',
    experience: 'Experience',
    skills: 'Skills',
    education: 'Education',
    present: 'present',
  },
  da: {
    online: 'Fuldt CV online:',
    experience: 'Erfaring',
    skills: 'Kompetencer',
    education: 'Uddannelse',
    present: 'nu',
  },
}

function sanitizeWinAnsi(text: string): string {
  return text.replaceAll('’', "'").replaceAll('“', '"').replaceAll('”', '"')
}

function wrapText(text: string, font: PDFFont, size: number, maxWidth: number): string[] {
  const words = sanitizeWinAnsi(text).split(/\s+/)
  const lines: string[] = []
  let current = ''

  for (const word of words) {
    const candidate = current ? `${current} ${word}` : word
    if (font.widthOfTextAtSize(candidate, size) <= maxWidth || !current) {
      current = candidate
    } else {
      lines.push(current)
      current = word
    }
  }
  if (current) lines.push(current)
  return lines
}

export async function renderCvPdf(locale: LocaleCode): Promise<Uint8Array> {
  const cv = getCv(locale)
  const labels = PDF_LABELS[locale]
  const cvOnlineUrl = locale === 'da' ? `${cv.siteUrl}/da/cv/` : `${cv.siteUrl}/cv/`

  const doc = await PDFDocument.create()
  doc.setTitle(`${cv.site.name} — CV`)
  doc.setAuthor(cv.site.name)
  doc.setSubject(cv.site.title)

  const regular = await doc.embedFont(StandardFonts.Helvetica)
  const bold = await doc.embedFont(StandardFonts.HelveticaBold)
  const ink = rgb(0.07, 0.07, 0.07)
  const muted = rgb(0.35, 0.35, 0.35)

  let page: PDFPage = doc.addPage([PAGE_WIDTH, PAGE_HEIGHT])
  let y = PAGE_HEIGHT - MARGIN_TOP

  function ensureSpace(height: number) {
    if (y - height < MARGIN_BOTTOM) {
      page = doc.addPage([PAGE_WIDTH, PAGE_HEIGHT])
      y = PAGE_HEIGHT - MARGIN_TOP
    }
  }

  function drawLines(
    text: string,
    opts: {
      font?: PDFFont
      size?: number
      lineHeight?: number
      color?: ReturnType<typeof rgb>
      indent?: number
      gapAfter?: number
      prefix?: string
    } = {},
  ) {
    const font = opts.font ?? regular
    const size = opts.size ?? 10
    const lineHeight = opts.lineHeight ?? size * 1.4
    const color = opts.color ?? ink
    const indent = opts.indent ?? 0
    const lines = wrapText(text, font, size, CONTENT_WIDTH - indent)

    for (let i = 0; i < lines.length; i++) {
      ensureSpace(lineHeight)
      if (i === 0 && opts.prefix) {
        page.drawText(opts.prefix, {
          x: MARGIN_X + indent - 10,
          y: y - size,
          size,
          font,
          color,
        })
      }
      page.drawText(lines[i]!, {
        x: MARGIN_X + indent,
        y: y - size,
        size,
        font,
        color,
      })
      y -= lineHeight
    }

    if (opts.gapAfter) {
      y -= opts.gapAfter
    }
  }

  function drawSectionHeading(title: string) {
    y -= 10
    ensureSpace(26)
    drawLines(title, { font: bold, size: 11.5, lineHeight: 16, gapAfter: 3 })
  }

  drawLines(cv.site.name, { font: bold, size: 20, lineHeight: 24, gapAfter: 2 })
  drawLines(cv.site.title, { size: 11, color: muted, lineHeight: 15, gapAfter: 4 })
  drawLines(
    `${cv.contact.town}, ${cv.contact.postcode} · ${cv.contact.phone} · ${cv.contact.email}`,
    { size: 9, color: muted, lineHeight: 13 },
  )
  drawLines(`${labels.online} ${cvOnlineUrl}`, { size: 9, color: muted, lineHeight: 13 })
  drawLines(`${cv.site.links.github} · ${cv.site.links.linkedin}`, {
    size: 9,
    color: muted,
    lineHeight: 13,
    gapAfter: 10,
  })

  for (const paragraph of cv.intro) {
    drawLines(paragraph, { size: 10, lineHeight: 14, gapAfter: 5 })
  }

  drawSectionHeading(labels.experience)
  for (const role of cv.roles) {
    const startLabel = formatCvMonth(role.start, locale)
    const endLabel = role.end ? formatCvMonth(role.end, locale) : labels.present
    ensureSpace(44)
    drawLines(`${role.title} · ${role.company}`, { font: bold, size: 10.5, lineHeight: 14.5 })
    drawLines(`${startLabel} – ${endLabel} — ${role.location}`, {
      size: 8.5,
      color: muted,
      lineHeight: 12.5,
      gapAfter: 2,
    })
    drawLines(role.summary, { size: 10, lineHeight: 14, gapAfter: 2 })

    for (const achievement of role.achievements) {
      drawLines(achievement, {
        size: 9.5,
        lineHeight: 13.5,
        indent: 14,
        prefix: '•',
        gapAfter: 1.5,
      })
    }

    if (role.note) {
      drawLines(role.note, { size: 8.8, color: muted, lineHeight: 12.5, gapAfter: 2 })
    }
    y -= 6
  }

  drawSectionHeading(labels.skills)
  for (const skill of cv.skills) {
    drawLines(skill.name, {
      size: 9.8,
      lineHeight: 14,
      indent: 14,
      prefix: '•',
      gapAfter: 1.5,
    })
  }

  drawSectionHeading(labels.education)
  if (cv.educationNote) {
    drawLines(cv.educationNote, { size: 9.8, color: muted, lineHeight: 14, gapAfter: 4 })
  }
  for (const edu of cv.education) {
    const startLabel = formatCvMonth(edu.start, locale)
    const endLabel = edu.end ? formatCvMonth(edu.end, locale) : labels.present
    ensureSpace(34)
    drawLines(`${edu.title} · ${edu.school}`, { font: bold, size: 10.5, lineHeight: 14.5 })
    drawLines(`${startLabel} – ${endLabel} — ${edu.location}`, {
      size: 8.5,
      color: muted,
      lineHeight: 12.5,
      gapAfter: 2,
    })
    if (edu.note) {
      drawLines(edu.note, { size: 9.5, lineHeight: 13.5, gapAfter: 2 })
    }
    y -= 4
  }

  return doc.save()
}
