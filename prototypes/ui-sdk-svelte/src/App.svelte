<!--
  Coque de l'IDE, pour évaluer UI SDK : barre de menus, palette de commandes (Ctrl+K),
  panneaux redimensionnables, arbre du projet, onglets de l'inspecteur.
  Les données viennent du cœur Rust (src/data.json, produit par `npm run data`).
-->
<script>
  import { Command, CommandEmpty, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList } from "@ui-sdk/svelte/command"
  import { Dialog, DialogBackdrop, DialogContent, DialogShortcut, DialogTitle, DialogTrigger } from "@ui-sdk/svelte/dialog"
  import { Menu, MenuContent, MenuItem, MenuSeparator, MenuTrigger } from "@ui-sdk/svelte/menu"
  import { Menubar } from "@ui-sdk/svelte/menubar"
  import { Resizable, ResizableHandle, ResizablePanel } from "@ui-sdk/svelte/resizable"
  import { Tabs, TabsList, TabsPanel, TabsTab } from "@ui-sdk/svelte/tabs"
  import { Tree, TreeGroup, TreeItem, TreeItemIndicator, TreeItemLabel } from "@ui-sdk/svelte/tree"
  import { agent } from "./agent.js"
  import data from "./data.json"
  import Orbit from "./Orbit.svelte"

  const phases = data.schedule.phases ?? []
  const contracts = data.contracts
  const components = data.components
  const archetypes = data.archetypes

  let treeValue = $state("system:Damage")
  let paletteOpen = $state(false)
  let theme = $state("dark")
  let agentInfo = $state("")

  const selectedSystem = $derived(treeValue?.startsWith("system:") ? treeValue.slice(7) : null)
  const selectedComponent = $derived(treeValue?.startsWith("component:") ? treeValue.slice(10) : null)
  const selectedPhase = $derived(treeValue?.startsWith("phase:") ? treeValue.slice(6) : null)
  const contract = $derived(contracts.find((c) => c.name === selectedSystem))
  const component = $derived(components.find((c) => c.name === selectedComponent))
  const stageOf = (name) => data.plan.stages.find((s) => s.systems.some((x) => x.name === name))?.stage

  $effect(() => {
    document.documentElement.dataset.theme = theme
  })

  const systemsByPhase = phases.map((phase) => ({
    phase,
    systems: data.plan.stages.flatMap((s) => s.systems).filter((s) => s.phase === phase),
  }))

  function run(value) {
    if (value === "theme-dark" || value === "theme-light") theme = value.slice(6)
    if (value === "palette") paletteOpen = true
  }

  function goto(value) {
    treeValue = value
    paletteOpen = false
  }

  function snapshot() {
    const parts = agent.snapshot()
    agentInfo = `${parts.length} composants exposés à l'agent`
    console.log(parts)
  }

  const accessLabel = {
    read: "lecture",
    write: "écriture",
    optional: "lecture optionnelle",
    with: "présence exigée",
    without: "absence exigée",
  }

  const structuralRows = (s) =>
    [
      ...(s.add ?? []).map((c) => ["ajoute", c]),
      ...(s.may_add ?? []).map((c) => ["peut ajouter", c]),
      ...(s.remove ?? []).map((c) => ["retire", c]),
      ...(s.may_remove ?? []).map((c) => ["peut retirer", c]),
      ...(s.spawn ?? []).map((c) => ["crée", c]),
      ...(s.may_spawn ?? []).map((c) => ["peut créer", c]),
      ...(s.despawn ? [["détruit", "l'entité"]] : []),
      ...(s.may_despawn ? [["peut détruire", "l'entité"]] : []),
    ]
</script>

<div class="app">
  <header class="topbar">
    <span class="brand">ECS NIDE</span>
    <Menubar aria-label="Application">
      <Menu onAction={run}>
        <MenuTrigger>Fichier</MenuTrigger>
        <MenuContent>
          <MenuItem value="open">Ouvrir un projet…</MenuItem>
          <MenuItem value="save">Enregistrer</MenuItem>
        </MenuContent>
      </Menu>
      <Menu onAction={run}>
        <MenuTrigger>Projet</MenuTrigger>
        <MenuContent>
          <MenuItem value="validate">Valider</MenuItem>
          <MenuItem value="plan">Calculer le plan</MenuItem>
          <MenuSeparator />
          <MenuItem value="palette">Aller à…</MenuItem>
        </MenuContent>
      </Menu>
      <Menu onAction={run}>
        <MenuTrigger>Affichage</MenuTrigger>
        <MenuContent>
          <MenuItem value="theme-dark">Thème sombre</MenuItem>
          <MenuItem value="theme-light">Thème clair</MenuItem>
        </MenuContent>
      </Menu>
    </Menubar>

    <Dialog bind:open={paletteOpen}>
      <DialogTrigger shortcut="mod+k" class="search">
        Aller à… <DialogShortcut />
      </DialogTrigger>
      <DialogBackdrop />
      <DialogContent class="palette">
        <DialogTitle>Aller à…</DialogTitle>
        <Command onAction={goto}>
          <CommandInput aria-label="Rechercher un élément" placeholder="Système, composant…" />
          <CommandEmpty>Aucun résultat.</CommandEmpty>
          <CommandList>
            <CommandGroup>
              <CommandGroupLabel>Systèmes</CommandGroupLabel>
              {#each contracts as c (c.name)}
                <CommandItem value={"system:" + c.name}>{c.name}</CommandItem>
              {/each}
            </CommandGroup>
            <CommandGroup>
              <CommandGroupLabel>Composants</CommandGroupLabel>
              {#each components as c (c.name)}
                <CommandItem value={"component:" + c.name}>{c.name}</CommandItem>
              {/each}
            </CommandGroup>
          </CommandList>
        </Command>
      </DialogContent>
    </Dialog>
  </header>

  <main class="workspace">
    <Resizable autoSaveId="ecs-nide-prototype">
      <ResizablePanel id="explorer" defaultSize={20} minSize={14}>
        <section class="panel">
          <h2>{data.manifest.name}</h2>
          <Tree aria-label="Éléments du projet" bind:value={treeValue} defaultExpanded={["systems", ...phases.map((p) => "phase:" + p)]}>
            <TreeItem value="systems">
              <TreeItemLabel><TreeItemIndicator />Systèmes</TreeItemLabel>
              <TreeGroup>
                {#each systemsByPhase as group (group.phase)}
                  <TreeItem value={"phase:" + group.phase}>
                    <TreeItemLabel><TreeItemIndicator />{group.phase}</TreeItemLabel>
                    <TreeGroup>
                      {#each group.systems as s (s.name)}
                        <TreeItem value={"system:" + s.name}>
                          <TreeItemLabel>{s.name}</TreeItemLabel>
                        </TreeItem>
                      {/each}
                    </TreeGroup>
                  </TreeItem>
                {/each}
              </TreeGroup>
            </TreeItem>
            <TreeItem value="components">
              <TreeItemLabel><TreeItemIndicator />Composants</TreeItemLabel>
              <TreeGroup>
                {#each components as c (c.name)}
                  <TreeItem value={"component:" + c.name}>
                    <TreeItemLabel>{c.name}</TreeItemLabel>
                  </TreeItem>
                {/each}
              </TreeGroup>
            </TreeItem>
            <TreeItem value="archetypes">
              <TreeItemLabel><TreeItemIndicator />Archétypes</TreeItemLabel>
              <TreeGroup>
                {#each archetypes as a (a.name)}
                  <TreeItem value={"archetype:" + a.name}>
                    <TreeItemLabel>{a.name}</TreeItemLabel>
                  </TreeItem>
                {/each}
              </TreeGroup>
            </TreeItem>
          </Tree>
        </section>
      </ResizablePanel>

      <ResizableHandle aria-label="Redimensionner l'explorateur" />

      <ResizablePanel id="orbit" defaultSize={50} minSize={30}>
        <Orbit
          plan={data.plan}
          {phases}
          {archetypes}
          selected={selectedSystem}
          onselect={(name) => (treeValue = "system:" + name)}
        />
      </ResizablePanel>

      <ResizableHandle aria-label="Redimensionner l'inspecteur" />

      <ResizablePanel id="inspector" defaultSize={30} minSize={20}>
        <section class="panel">
          {#if contract}
            <h2>{contract.name}</h2>
            <p class="meta">
              Phase <strong>{contract.phase ?? "—"}</strong> · étape <strong>{stageOf(contract.name)}</strong>
              · {contract.per_entity ? "par entité" : "global"} · une boucle sur {contract.every}
            </p>
            <p>{contract.description}</p>
            <Tabs defaultValue="contract">
              <TabsList aria-label="Détails du système">
                <TabsTab value="contract">Contrat</TabsTab>
                <TabsTab value="spec">Spécification</TabsTab>
                <TabsTab value="examples">Exemples ({contract.examples.length})</TabsTab>
              </TabsList>
              <TabsPanel value="contract">
                <table>
                  <thead><tr><th>Donnée</th><th>Accès</th></tr></thead>
                  <tbody>
                    {#each Object.entries(contract.components) as [name, access] (name)}
                      <tr><td>{name}</td><td class="access {access}">{accessLabel[access]}</td></tr>
                    {/each}
                    {#each Object.entries(contract.resources) as [name, access] (name)}
                      <tr><td>{name} <em>(ressource)</em></td><td class="access {access}">{accessLabel[access]}</td></tr>
                    {/each}
                    {#each structuralRows(contract.structural) as [what, name] (what + name)}
                      <tr><td>{name}</td><td class="access structural">{what}</td></tr>
                    {/each}
                  </tbody>
                </table>
                {#if contract.after.length}
                  <p class="meta">S'exécute après : {contract.after.join(", ")}</p>
                {/if}
              </TabsPanel>
              <TabsPanel value="spec">
                <pre>{contract.spec}</pre>
              </TabsPanel>
              <TabsPanel value="examples">
                {#each contract.examples as example (example.id)}
                  <h3>{example.id}</h3>
                  <pre>{JSON.stringify(example, null, 2)}</pre>
                {/each}
              </TabsPanel>
            </Tabs>
          {:else if component}
            <h2>{component.name}</h2>
            <p>{component.description}</p>
            {#if Object.keys(component.fields ?? {}).length}
              <table>
                <thead><tr><th>Champ</th><th>Type</th><th>Contraintes</th></tr></thead>
                <tbody>
                  {#each Object.entries(component.fields) as [name, field] (name)}
                    <tr>
                      <td>{name}</td>
                      <td>{field.type}</td>
                      <td>{[field.min !== undefined && `min ${field.min}`, field.max !== undefined && `max ${field.max}`, field.default !== undefined && `défaut ${JSON.stringify(field.default)}`].filter(Boolean).join(" · ")}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {:else}
              <p class="meta">Marqueur (aucun champ).</p>
            {/if}
          {:else if selectedPhase}
            <h2>Phase {selectedPhase}</h2>
            <table>
              <thead><tr><th>Étape</th><th>Systèmes</th></tr></thead>
              <tbody>
                {#each data.plan.stages.filter((s) => s.systems.some((x) => x.phase === selectedPhase)) as stage (stage.stage)}
                  <tr><td>{stage.stage}</td><td>{stage.systems.map((x) => x.name).join(", ")}</td></tr>
                {/each}
              </tbody>
            </table>
          {:else}
            <p class="meta">Sélectionne un système, une phase ou un composant.</p>
          {/if}
        </section>
      </ResizablePanel>
    </Resizable>
  </main>

  <footer class="statusbar">
    <span>{data.diagnostics.length} diagnostic(s)</span>
    <span>profil {data.plan.profile.name}</span>
    <span>{data.plan.stages.length} étapes · {contracts.length} systèmes</span>
    <button type="button" onclick={snapshot}>Instantané agent</button>
    <span>{agentInfo}</span>
  </footer>
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100%;
  }

  .topbar {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.25rem 0.75rem;
    border-bottom: 1px solid var(--ui-border);
    background: var(--nide-panel);
  }

  .brand {
    font-weight: 700;
    letter-spacing: 0.04em;
  }

  .topbar :global(.search) {
    margin-inline-start: auto;
    min-width: 14rem;
    justify-content: space-between;
  }

  :global(.palette) {
    width: min(36rem, 90vw);
  }

  .workspace {
    min-height: 0;
  }

  .workspace :global([data-slot="resizable"]) {
    height: 100%;
  }

  .panel {
    height: 100%;
    overflow: auto;
    padding: 0.75rem;
    box-sizing: border-box;
    background: var(--nide-panel);
  }

  h2 {
    margin: 0 0 0.5rem;
    font-size: 1rem;
  }

  h3 {
    margin: 0.75rem 0 0.25rem;
    font-size: 0.875rem;
  }

  .meta {
    color: var(--ui-muted-foreground);
  }

  table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 0.5rem;
  }

  th,
  td {
    text-align: start;
    padding: 0.25rem 0.375rem;
    border-bottom: 1px solid var(--ui-border);
  }

  th {
    color: var(--ui-muted-foreground);
    font-weight: 500;
  }

  .access.write {
    color: var(--nide-phase-4);
  }

  .access.read,
  .access.optional {
    color: var(--nide-phase-1);
  }

  .access.structural {
    color: var(--nide-phase-5);
  }

  pre {
    white-space: pre-wrap;
    font-family: var(--ui-font-mono);
    font-size: 0.75rem;
    background: var(--ui-muted);
    padding: 0.5rem;
    border-radius: var(--ui-radius);
  }

  .statusbar {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 0.25rem 0.75rem;
    border-top: 1px solid var(--ui-border);
    color: var(--ui-muted-foreground);
    background: var(--nide-panel);
  }

  .statusbar button {
    font: inherit;
    color: inherit;
    background: none;
    border: 1px solid var(--ui-border);
    border-radius: var(--ui-radius);
    padding: 0 0.5rem;
    cursor: pointer;
  }
</style>
