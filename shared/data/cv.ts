export type LocaleCode = 'en' | 'da'

export interface CvRole {
  start: string
  end?: string | null
  title: string
  company: string
  location: string
  summary: string
  achievements: string[]
  note?: string | null
}

export interface CvEducation {
  start: string
  end?: string | null
  title: string
  school: string
  location: string
  note?: string | null
}

export interface CvData {
  locale: LocaleCode
  siteUrl: string
  pdfPath: string
  intro: string[]
  about: string[]
  educationNote: string | null
  site: {
    name: string
    title: string
    stack: string[]
    available: boolean
    availableNote: string
    links: {
      github: string
      linkedin: string
      email: string
    }
  }
  contact: {
    town: string
    postcode: string
    phone: string
    phoneHref: string
    email: string
  }
  roles: CvRole[]
  skills: { name: string }[]
  education: CvEducation[]
}

const MONTHS: Record<LocaleCode, readonly string[]> = {
  en: [
    'January',
    'February',
    'March',
    'April',
    'May',
    'June',
    'July',
    'August',
    'September',
    'October',
    'November',
    'December',
  ],
  da: [
    'januar',
    'februar',
    'marts',
    'april',
    'maj',
    'juni',
    'juli',
    'august',
    'september',
    'oktober',
    'november',
    'december',
  ],
}

export function normalizeLocale(value?: string | null): LocaleCode {
  return value === 'da' ? 'da' : 'en'
}

export function formatCvMonth(value: string, locale: LocaleCode = 'en'): string {
  const match = /^(\d{4})-(\d{2})$/.exec(value)
  if (!match) {
    throw new Error(`Invalid CV date "${value}" (expected YYYY-MM)`)
  }
  const year = Number(match[1])
  const monthIndex = Number(match[2]) - 1
  if (monthIndex < 0 || monthIndex > 11) {
    throw new Error(`Invalid CV date "${value}" (month out of range)`)
  }
  return `${MONTHS[locale][monthIndex]} ${year}`
}

const SHARED_SITE = {
  name: 'Jonas Hansen',
  stack: ['PHP', 'Laravel', 'Symfony', 'Go', 'Rust', 'Nuxt', 'React', 'Kubernetes', 'FreeBSD'],
  available: true,
  links: {
    github: 'https://github.com/sasin91',
    linkedin: 'https://www.linkedin.com/in/jonas-hansen-2b6828110',
    email: 'mailto:jonas.kerwin.hansen@gmail.com',
  },
}

const SHARED_CONTACT = {
  town: 'Slagelse',
  postcode: '4200',
  phone: '+45 50106917',
  phoneHref: '+4550106917',
  email: 'jonas.kerwin.hansen@gmail.com',
}

const CV_BY_LOCALE: Record<LocaleCode, CvData> = {
  en: {
    locale: 'en',
    siteUrl: 'https://sasin91.xyz',
    pdfPath: '/cv_jonas_hansen_software_developer.pdf',
    intro: [
      'I have been working with PHP and Laravel since 2015 and since then I have helped develop a ticketing agency, a video streaming platform and a recruitment platform.',
      'I enjoy designing and delivering customized and solid solutions, but also enjoy support and listening to user requests and issues with a smile.',
    ],
    about: [
      'I enjoy both big and small tasks. I have taken full projects from start to finish and maintained them afterwards.',
      'In general I push for the simplest solution and aim for long-term stability. That is roughly why AthletOS runs on one FreeBSD box I can roll back when I break something, and why this site stays small, fast and self-contained.',
      'Based in Slagelse. PHP and Laravel since 2015 — a ticketing agency, a video streaming platform, a recruitment platform — with Symfony, Go, Kubernetes and Rust picked up along the way, where the problem asked for them.',
      'I write here when something seems worth writing down. Usually after it broke, and usually with the numbers.',
    ],
    educationNote: 'Two short-cycle higher educations.',
    site: {
      ...SHARED_SITE,
      title: 'Software developer',
      availableNote: 'Available for work',
    },
    contact: SHARED_CONTACT,
    roles: [
      {
        start: '2024-09',
        end: '2026-02',
        title: 'Web Developer',
        company: 'JUICE ApS',
        location: 'Copenhagen',
        summary: 'Job & candidate matchmaking platform',
        achievements: [
          'Upgraded the Symfony framework from version 6 to 7',
          'Built a candidate ranking and sorting engine, enabling fast, relevancy-based candidate search',
          'Integrated AI to automatically generate SmartMatch job posts from uploaded job ads',
          'Led development in close collaboration with the CTO, incorporating regular feedback and code review',
        ],
        note: 'Rehired by JUICE after the 2023 round of cuts. The role ended in February 2026 when the company again cut staff for financial reasons.',
      },
      {
        start: '2023-09',
        end: '2024-09',
        title: 'Web Developer',
        company: 'Supeo',
        location: 'Næstved',
        summary: 'Web development agency',
        achievements: [
          'Owned Laravel development end-to-end, including customer interactions and development of SamFocus in Laravel 9',
          'Provided code review and development support across other work, including Supeo Flex in React, Express.js & GraphQL',
        ],
      },
      {
        start: '2023-01',
        end: '2023-08',
        title: 'Web Developer',
        company: 'JUICE ApS',
        location: 'Copenhagen',
        summary: 'Job & candidate matchmaking platform',
        achievements: [
          'Introduced CI/CD enabling efficient and fast delivery of Symfony 6',
          'Delivered multiple features using Symfony, Twig, and Stimulus.js',
          'Worked in close collaboration with the CTO, leading development while incorporating feedback and code review',
        ],
        note: 'The role ended in August 2023 when JUICE cut staff for financial reasons. Rehired by the company in September 2024.',
      },
      {
        start: '2020-04',
        end: '2023-01',
        title: 'Tech Lead',
        company: 'Syncronet',
        location: 'Slagelse',
        summary: 'Live streaming social media platform',
        achievements: [
          'Led migration from an expensive setup on Azure using media services to Linode with mux.com for video delivery',
          'Set up, managed, and monitored VPS infrastructure, including DNS and Let\'s Encrypt SSL',
          'Architected a Kubernetes cluster with Go services and Nginx endpoints wrapping FFmpeg for internal video processing and delivery',
          'Debugged slow database queries and implemented caching to keep the platform performant',
          'Led frontend development in Nuxt 3 and mobile development in React Native while maintaining the Laravel backend',
        ],
      },
      {
        start: '2017-01',
        end: '2020-02',
        title: 'Lead Developer',
        company: 'GHC Travel / Iraqi Airways',
        location: 'Copenhagen',
        summary: 'Airline booking platform',
        achievements: [
          'Rebuilt a procedural PHP platform in Laravel within the first year, after joining as a junior developer',
          'Progressed from junior to senior developer, took over as sole developer after a teammate\'s departure, and led development in the final phase, continuously building out the platform throughout',
          'Migrated from a traditional PHP web host to VPS servers, setting up, managing, and monitoring infrastructure including DNS and Let\'s Encrypt SSL',
          'Maintained and upgraded the Laravel platform up to version 8, including a comprehensive PHPUnit test suite ensuring fast feedback loops on changes',
          'Debugged slow database queries and implemented caching to keep the platform responsive',
          'Resolved growing domain-boundary issues by extracting the frontend into 3 distinct Nuxt 2 apps sharing UI and design packages over npm and consuming a Laravel API, and refactored the backend using Domain-Driven Design (DDD)',
          'Automated ticket booking and flight monitoring using UiPath RPA in collaboration with a new team member, improving ticket sales and customer satisfaction',
          'Provided live debugging and triage support directly to reception staff and agents, acting as first-line support',
        ],
        note: 'The role ended in February 2020 when the company wound down, after COVID-19 shut down air travel.',
      },
    ],
    skills: [
      { name: 'Linux server management and maintenance' },
      { name: 'Web & App development' },
      { name: 'Database administration' },
    ],
    education: [
      {
        start: '2014-02',
        end: '2015-08',
        title: 'Web integrator',
        school: 'Roskilde Teknisk Skole',
        location: 'Høng',
      },
      {
        start: '2012-01',
        end: '2013-08',
        title: 'Strøm, styring & IT',
        school: 'Selandia CEU',
        location: 'Slagelse',
        note: 'Including Cisco CCNA and IP based network management.',
      },
    ],
  },
  da: {
    locale: 'da',
    siteUrl: 'https://sasin91.xyz',
    pdfPath: '/da/cv_jonas_hansen_software_developer.pdf',
    intro: [
      'Jeg har arbejdet med PHP og Laravel siden 2015 og har siden været med til at udvikle et billetbureau, en videostreamingplatform og en rekrutteringsplatform.',
      'Jeg sætter pris på at designe og levere skræddersyede og solide løsninger, men nyder også support og at lytte til brugernes ønsker og udfordringer med et smil.',
    ],
    about: [
      'Jeg holder af både store og små opgaver. Jeg har taget hele projekter fra start til slut og vedligeholdt dem efterfølgende.',
      'Generelt går jeg efter den enkleste løsning og sigter mod langsigtet stabilitet. Det er groft sagt derfor AthletOS kører på én FreeBSD-maskine, jeg kan rulle tilbage når jeg ødelægger noget, og derfor denne side holdes lille, hurtig og selvforsynende.',
      'Bosat i Slagelse. PHP og Laravel siden 2015 — et billetbureau, en videostreamingplatform, en rekrutteringsplatform — med Symfony, Go, Kubernetes og Rust samlet op undervejs, hvor problemet bad om det.',
      'Jeg skriver her, når noget virker værd at skrive ned. Som regel efter det gik i stykker, og som regel med tallene på.',
    ],
    educationNote: 'To korte videregående / erhvervsrettede uddannelser.',
    site: {
      ...SHARED_SITE,
      title: 'Softwareudvikler',
      availableNote: 'Ledig til nye opgaver',
    },
    contact: SHARED_CONTACT,
    roles: [
      {
        start: '2024-09',
        end: '2026-02',
        title: 'Webudvikler',
        company: 'JUICE ApS',
        location: 'København',
        summary: 'Platform til job- og kandidatmatchning',
        achievements: [
          'Opgraderede Symfony-frameworket fra version 6 til 7',
          'Byggede en kandidat-rangerings- og sorteringsmotor, som muliggjorde hurtig, relevansbaseret kandidatsøgning',
          'Integrerede AI til automatisk at generere SmartMatch-jobopslag ud fra uploadede jobannoncer',
          'Ledte udviklingen i tæt samarbejde med CTO’en med løbende feedback og code review',
        ],
        note: 'Genansat hos JUICE efter sparerunden i 2023. Stillingen sluttede i februar 2026, da virksomheden igen skar i bemandingen af økonomiske årsager.',
      },
      {
        start: '2023-09',
        end: '2024-09',
        title: 'Webudvikler',
        company: 'Supeo',
        location: 'Næstved',
        summary: 'Webbureau',
        achievements: [
          'Ejede Laravel-udviklingen fra start til slut, inklusiv kundekontakt og udvikling af SamFocus i Laravel 9',
          'Bidrog med code review og udviklingsstøtte på øvrige opgaver, herunder Supeo Flex i React, Express.js & GraphQL',
        ],
      },
      {
        start: '2023-01',
        end: '2023-08',
        title: 'Webudvikler',
        company: 'JUICE ApS',
        location: 'København',
        summary: 'Platform til job- og kandidatmatchning',
        achievements: [
          'Indførte CI/CD til effektiv og hurtig levering af Symfony 6',
          'Leverede flere nye funktioner med Symfony, Twig og Stimulus.js',
          'Arbejdede i tæt samarbejde med CTO’en og drev udviklingen med løbende feedback og code review',
        ],
        note: 'Stillingen sluttede i august 2023, da JUICE skar i bemandingen af økonomiske årsager. Genansat af virksomheden i september 2024.',
      },
      {
        start: '2020-04',
        end: '2023-01',
        title: 'Tech Lead',
        company: 'Syncronet',
        location: 'Slagelse',
        summary: 'Social medieplatform til livestreaming',
        achievements: [
          'Ledte migreringen fra en dyr Azure Media Services-opsætning til Linode med mux.com til videolevering',
          'Opsatte, administrerede og overvågede VPS-infrastruktur inklusiv DNS og Let’s Encrypt SSL',
          'Arkitekterede et Kubernetes-kluster med Go-services og Nginx-endpoints omkring FFmpeg til intern videobehandling og -levering',
          'Fejlsøgte langsomme databaseforespørgsler og indførte caching for at holde platformen hurtig',
          'Ledte frontend-udviklingen i Nuxt 3 og mobiludviklingen i React Native sideløbende med vedligeholdelse af Laravel-backenden',
        ],
      },
      {
        start: '2017-01',
        end: '2020-02',
        title: 'Lead Developer',
        company: 'GHC Travel / Iraqi Airways',
        location: 'København',
        summary: 'Flybookingplatform',
        achievements: [
          'Genopbyggede en procedural PHP-platform i Laravel inden for det første år efter ansættelse som juniorudvikler',
          'Voksede fra junior- til seniorudvikler, overtog rollen som eneste udvikler efter en kollegas afgang og ledte udviklingen i den afsluttende fase',
          'Migrerede fra traditionelt PHP-webhotel til VPS-servere med opsætning, drift og overvågning inklusiv DNS og Let’s Encrypt SSL',
          'Vedligeholdt og opgraderede Laravel-platformen op til version 8 inklusiv en omfattende PHPUnit-testsuite med hurtig feedback ved ændringer',
          'Fejlsøgte langsomme databaseforespørgsler og indførte caching for at sikre høj responstid',
          'Løste voksende domænegrænse-udfordringer ved at udskille frontenden i 3 selvstændige Nuxt 2-apps med delte UI- og designpakker over npm mod et Laravel-API, samt refaktorerede backenden med Domain-Driven Design (DDD)',
          'Automatiserede billetbestilling og flyovervågning med UiPath RPA i samarbejde med en ny kollega, hvilket forbedrede billetsalg og kundetilfredshed',
          'Ydede direkte fejlfinding og førstelinjessupport til receptionister og rejseagenter',
        ],
        note: 'Stillingen sluttede i februar 2020, da virksomheden lukkede ned efter COVID-19 satte luftfarten i stå.',
      },
    ],
    skills: [
      { name: 'Linux-serveradministration og vedligeholdelse' },
      { name: 'Web- og appudvikling' },
      { name: 'Databaseadministration' },
    ],
    education: [
      {
        start: '2014-02',
        end: '2015-08',
        title: 'Webintegrator',
        school: 'Roskilde Teknisk Skole',
        location: 'Høng',
      },
      {
        start: '2012-01',
        end: '2013-08',
        title: 'Strøm, styring & IT',
        school: 'Selandia CEU',
        location: 'Slagelse',
        note: 'Inklusiv Cisco CCNA og IP-baseret netværksadministration.',
      },
    ],
  },
}

export function getCv(locale: string = 'en'): CvData {
  return CV_BY_LOCALE[normalizeLocale(locale)]
}

export const cv: CvData = CV_BY_LOCALE.en
