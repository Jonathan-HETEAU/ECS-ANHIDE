const { chromium } = require("playwright")
;(async () => {
  const out = process.argv[2] || "."
  const browser = await chromium.launch()
  const page = await browser.newPage({ viewport: { width: 1440, height: 860 }, deviceScaleFactor: 2 })
  const errors = []
  page.on("pageerror", (e) => errors.push(e.message))
  page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()) })
  for (let i = 0; i < 40; i++) {
    try { await page.goto("http://localhost:4173", { waitUntil: "networkidle" }); break } catch { await new Promise(r => setTimeout(r, 250)) }
  }
  await page.waitForSelector("svg [role=button]")
  await page.screenshot({ path: `${out}/1-sombre.png` })

  // Clic sur une planète : l'inspecteur suit.
  await page.getByRole("button", { name: /Système Movement/ }).locator("circle").click()
  const inspector = await page.locator("section.panel h2").nth(1).textContent()

  // Navigation clavier dans l'arbre.
  await page.getByRole("treeitem", { name: "Movement" }).click()
  await page.keyboard.press("ArrowDown")
  await page.keyboard.press("Enter")
  const afterKeys = await page.locator("section.panel h2").nth(1).textContent()

  // Onglet Exemples.
  await page.getByRole("tab", { name: /Exemples/ }).click()
  await page.screenshot({ path: `${out}/2-exemples.png` })

  // Palette de commandes (Ctrl+K), recherche, choix.
  await page.keyboard.press("Control+k")
  await page.getByRole("combobox", { name: "Rechercher un élément" }).fill("heal")
  await page.screenshot({ path: `${out}/3-palette.png` })
  await page.keyboard.press("Enter")
  const fromPalette = await page.locator("section.panel h2").nth(1).textContent()

  // Thème clair via la barre de menus, au clavier.
  await page.getByRole("menuitem", { name: "Affichage" }).click()
  await page.getByRole("menuitem", { name: "Thème clair" }).click()
  await page.getByRole("button", { name: /Système Damage/ }).locator("circle").click()
  await page.screenshot({ path: `${out}/4-clair.png` })

  // Pont pour agents.
  const snapshot = await page.evaluate(() => window.agent.snapshot())
  const kinds = {}
  for (const c of snapshot) kinds[c.component] = (kinds[c.component] || 0) + 1
  const tree = snapshot.find((c) => c.component === "tree")
  console.log(JSON.stringify({ inspector, afterKeys, fromPalette, agentComponents: snapshot.length, kinds, treeActions: tree && tree.actions, treeValue: tree && tree.value, errors }, null, 1))
  await browser.close()
})()
