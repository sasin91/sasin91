import { createAgent, registerTools } from '@ui-sdk/vue/agent'

export default defineNuxtPlugin(() => {
  const agent = createAgent({
    allow: (step) => !step.confirm,
  })
  registerTools(agent, { allow: () => true })
  return {
    provide: { uiAgent: agent },
  }
})
