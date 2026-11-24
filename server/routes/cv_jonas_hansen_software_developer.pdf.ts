import { normalizeLocale } from '#shared/data/cv'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const locale = normalizeLocale(typeof query.lang === 'string' ? query.lang : 'en')
  const bytes = await renderCvPdf(locale)
  setHeader(event, 'content-type', 'application/pdf')
  setHeader(
    event,
    'content-disposition',
    `inline; filename="cv_jonas_hansen_software_developer${locale === 'da' ? '_da' : ''}.pdf"`,
  )
  return bytes
})
