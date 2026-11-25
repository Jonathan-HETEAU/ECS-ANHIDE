// Pont UI SDK pour agents : une IA peut lire l'état des composants et appeler leurs actions.
// Exposé en `window.agent` pour l'essayer depuis la console du navigateur.
import { createAgent } from "@ui-sdk/svelte/agent"

const agent = createAgent()
Object.assign(window, { agent })

export { agent }
