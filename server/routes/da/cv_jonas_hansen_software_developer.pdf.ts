export default defineEventHandler(async (event) => {
  const bytes = await renderCvPdf('da')
  setHeader(event, 'content-type', 'application/pdf')
  setHeader(event, 'content-disposition', 'inline; filename="cv_jonas_hansen_software_developer_da.pdf"')
  return bytes
})
