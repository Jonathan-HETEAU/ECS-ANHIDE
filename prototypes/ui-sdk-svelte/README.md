# Prototype : UI SDK (Vercel Labs) pour l'interface d'ECS NIDE

> **Prototype jetable.** Il sert à évaluer une bibliothèque, pas à construire l'IDE : il n'est
> couvert par aucune exigence et ne fait pas partie du produit. L'IDE sera construit en phase 5,
> dans Tauri, à partir des exigences.

## Ce qu'il montre

La coque d'un IDE, avec les **vraies données** du projet d'exemple `space-shooter`, calculées par
le cœur Rust (plan d'exécution, contrats, diagnostics) :

- barre de menus (Fichier, Projet, Affichage) ;
- palette de commandes (`Ctrl+K`) pour aller à un système ou un composant ;
- trois panneaux redimensionnables, dont la disposition est retenue ;
- arbre du projet (systèmes rangés par phase, composants, archétypes), navigable au clavier ;
- vue orbitale en SVG : trou noir, une position par étape, orbites concentriques pour les
  systèmes parallèles, secteurs de phase, flottes en mouvement ;
- inspecteur à onglets : contrat, spécification, exemples ;
- thèmes sombre et clair, **sans Tailwind** : uniquement des variables CSS ;
- pont pour agents : `window.agent.snapshot()` dans la console du navigateur.

## Lancer

```sh
npm install
npm run data      # régénère src/data.json depuis examples/space-shooter (nécessite cargo)
npm run dev
```

Vérification automatique dans un navigateur (Playwright) :

```sh
npm run build && npx vite preview --port 4173 &
node scripts/check.cjs /chemin/des/captures
```

## Bilan de l'évaluation (`@ui-sdk/svelte` et `@ui-sdk/core` 0.1.0-beta.17)

**Points forts**

- **Svelte 5 natif** (Svelte 5.33 minimum) et 70 composants, qui couvrent tous les besoins d'un IDE :
  `resizable` (avec sauvegarde de la disposition), `tree` (clavier, chargement à la demande, lignes
  virtualisées), `command` (palette), `menubar`, `menu` (y compris menu contextuel), `tabs`,
  `dialog`, `toolbar`, `shortcuts`, `scroll-area`, `toast`, `tooltip`…
- **Sans style imposé.** La feuille par défaut (`@ui-sdk/core/web.css`) se pilote entièrement par
  des variables `--ui-*`, avec clair/sombre natif (`light-dark()`) et une densité adaptée à un IDE
  (texte de 13 px). Le thème d'ECS NIDE tient dans `src/theme.css` ; les mêmes variables colorent
  la vue orbitale.
- **Accessibilité et clavier** sans effort : vérifiés ici sur l'arbre (flèches, Entrée), la palette
  (`Ctrl+K`, recherche, Entrée) et la barre de menus.
- **Pont pour agents** : une IA peut lire l'état des composants et appeler leurs actions, avec un
  rappel `allow` pour approuver ou refuser chaque action (cohérent avec « l'IDE signale,
  l'utilisateur décide »). Il peut aussi servir aux tests de l'interface. Ici, 8 composants sont
  exposés ; l'arbre propose `setValue`, `expand`, `collapse`…
- Documentation livrée **dans le paquet** (`node_modules/@ui-sdk/svelte/docs`), à la version
  installée, plus un kit de test (`@ui-sdk/core/testing`).
- Taille raisonnable pour une application de bureau : 12 Ko de CSS et 128 Ko de JS compressés, pour
  tout le prototype.

**Risques**

- **Bêta** : la version 0.1.0-beta.17 a été publiée la veille de l'évaluation ; l'API peut changer
  d'ici la version stable. Il faut figer la version exacte.
- Projet récent de Vercel Labs : maturité et pérennité à surveiller.
- Avec les styles propres à un composant Svelte, cibler les parties d'un composant UI SDK demande
  `:global(...)` (documenté).
