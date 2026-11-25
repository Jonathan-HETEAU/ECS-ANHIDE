<!--
  Vue orbitale (esquisse en SVG ; la vraie vue utilisera PixiJS).
  - Le trou noir au centre, une position angulaire par étape du plan, la première en haut,
    les suivantes dans le sens horaire (VIS-001).
  - Les systèmes d'une même étape sont sur des orbites concentriques (VIS-002).
  - Les phases sont des secteurs colorés de l'orbite (VIS-010).
  - Des flottes (une par archétype) font le tour : un tour = une boucle.
-->
<script>
  let { plan, phases = [], archetypes = [], selected = null, onselect } = $props()

  const size = 640
  const center = size / 2
  const baseRadius = 175
  const orbitGap = 42

  const stageCount = $derived(plan.stages.length)
  const step = $derived(360 / Math.max(stageCount, 1))
  const angleOf = (index) => -90 + index * step

  function point(radius, angleDeg) {
    const a = (angleDeg * Math.PI) / 180
    return { x: center + radius * Math.cos(a), y: center + radius * Math.sin(a) }
  }

  function arc(radius, from, to) {
    const p0 = point(radius, from)
    const p1 = point(radius, to)
    const large = to - from > 180 ? 1 : 0
    return `M ${p0.x} ${p0.y} A ${radius} ${radius} 0 ${large} 1 ${p1.x} ${p1.y}`
  }

  const phaseColor = (phase) => {
    const index = phases.indexOf(phase)
    return index < 0 ? "var(--ui-muted-foreground)" : `var(--nide-phase-${(index % 5) + 1})`
  }

  const maxParallel = $derived(Math.max(1, ...plan.stages.map((s) => s.systems.length)))
  const outerRadius = $derived(baseRadius + (maxParallel - 1) * orbitGap)

  const planets = $derived(
    plan.stages.flatMap((stage, index) =>
      stage.systems.map((system, lane) => {
        const radius = baseRadius + lane * orbitGap
        const planet = point(radius, angleOf(index))
        // Seul sur son étape : étiquette sous la planète. Orbites parallèles : l'intérieure
        // vers le centre, les autres vers l'extérieur, alignées du côté opposé à la planète.
        let label = { x: planet.x, y: planet.y + 31, anchor: "middle" }
        if (stage.systems.length > 1) {
          const outward = lane > 0
          const p = point(outward ? radius + 24 : radius - 24, angleOf(index))
          const dx = p.x - planet.x
          const anchor = Math.abs(dx) < 6 ? "middle" : dx > 0 ? "start" : "end"
          label = { x: p.x, y: p.y + 4 + (anchor === "middle" ? (p.y > planet.y ? 8 : -4) : 0), anchor }
        }
        return { ...system, stage: stage.stage, ...planet, label }
      }),
    ),
  )

  const sectors = $derived(
    phases
      .map((phase) => {
        const indices = plan.stages
          .map((stage, i) => (stage.systems.some((s) => s.phase === phase) ? i : -1))
          .filter((i) => i >= 0)
        if (indices.length === 0) return null
        const from = angleOf(Math.min(...indices)) - step / 2 + 2
        const to = angleOf(Math.max(...indices)) + step / 2 - 2
        return { phase, from, to, label: point(outerRadius + 52, (from + to) / 2) }
      })
      .filter(Boolean),
  )

  function onkey(event, name) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault()
      onselect?.(name)
    }
  }
</script>

<svg viewBox="0 0 {size} {size}" role="group" aria-label="Vue orbitale du plan d'exécution">
  <defs>
    <radialGradient id="hole">
      <stop offset="0%" stop-color="black" />
      <stop offset="62%" stop-color="black" />
      <stop offset="70%" stop-color="oklch(0.75 0.17 60)" />
      <stop offset="78%" stop-color="oklch(0.55 0.2 300 / 0.6)" />
      <stop offset="100%" stop-color="transparent" />
    </radialGradient>
  </defs>

  {#each sectors as sector (sector.phase)}
    <path
      d={arc(outerRadius + 30, sector.from, sector.to)}
      stroke={phaseColor(sector.phase)}
      class="sector"
    />
    <text x={sector.label.x} y={sector.label.y} class="sector-label" fill={phaseColor(sector.phase)}>
      {sector.phase}
    </text>
  {/each}

  {#each Array(maxParallel) as _, lane}
    <circle cx={center} cy={center} r={baseRadius + lane * orbitGap} class="orbit" />
  {/each}

  <g class="fleets" style="transform-origin: {center}px {center}px">
    {#each archetypes as archetype, i (archetype.name)}
      {@const p = point(120, -90 - i * 14)}
      <circle cx={p.x} cy={p.y} r="4" class="ship">
        <title>Flotte {archetype.name}</title>
      </circle>
    {/each}
  </g>

  <circle cx={center} cy={center} r="70" fill="url(#hole)" />
  <text x={center} y={center + 4} class="hole-label">boucle</text>

  {#each planets as planet (planet.name)}
    <g
      class="planet"
      class:selected={planet.name === selected}
      role="button"
      tabindex="0"
      aria-label="Système {planet.name}, étape {planet.stage}"
      aria-pressed={planet.name === selected}
      onclick={() => onselect?.(planet.name)}
      onkeydown={(event) => onkey(event, planet.name)}
    >
      <circle cx={planet.x} cy={planet.y} r="15" fill={phaseColor(planet.phase)} />
      <text x={planet.label.x} y={planet.label.y} text-anchor={planet.label.anchor} class="planet-label">{planet.name}</text>
      <text x={planet.x} y={planet.y + 4} class="planet-stage">{planet.stage}</text>
    </g>
  {/each}
</svg>

<style>
  svg {
    width: 100%;
    height: 100%;
    background: var(--nide-space);
    font-family: var(--ui-font-sans);
  }

  .orbit {
    fill: none;
    stroke: var(--nide-orbit);
    stroke-dasharray: 2 6;
  }

  .sector {
    fill: none;
    stroke-width: 10;
    stroke-linecap: round;
    opacity: 0.35;
  }

  .sector-label {
    font-size: 13px;
    font-weight: 600;
    text-anchor: middle;
  }

  .hole-label {
    fill: oklch(0.85 0.05 60);
    font-size: 11px;
    text-anchor: middle;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .planet {
    cursor: pointer;
    outline: none;
  }

  .planet circle {
    stroke: var(--nide-space);
    stroke-width: 3;
    transition: r 120ms;
  }

  .planet:hover circle,
  .planet:focus-visible circle {
    r: 18;
  }

  .planet.selected circle {
    stroke: var(--ui-foreground);
    r: 19;
  }

  .planet-label {
    fill: var(--ui-foreground);
    font-size: 12px;
  }

  .planet-stage {
    fill: oklch(0.15 0 0);
    font-size: 11px;
    font-weight: 700;
    text-anchor: middle;
    pointer-events: none;
  }

  .ship {
    fill: var(--ui-foreground);
    opacity: 0.85;
  }

  .fleets {
    animation: orbit 24s linear infinite;
  }

  @keyframes orbit {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .fleets {
      animation: none;
    }
  }
</style>
