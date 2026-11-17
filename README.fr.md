# ZetaAnim

[English](README.md) | **Français**

[![Licence : MIT OU Apache-2.0](https://img.shields.io/badge/licence-MIT%20OU%20Apache--2.0-blue)](#licence)
[![Statut : en développement précoce](https://img.shields.io/badge/statut-d%C3%A9veloppement%20pr%C3%A9coce-orange)](#feuille-de-route)

**ZetaAnim est un moteur d'animation mathématique open source écrit en Rust, avec un éditeur visuel et un langage de script.**

> [!WARNING]
> ZetaAnim est en développement précoce et n'est pas encore utilisable. Les interfaces et le format des fichiers peuvent changer sans préavis.

## Présentation

ZetaAnim permet de construire des figures mathématiques et de les animer, soit à la souris dans un éditeur avec aperçu en direct, soit en Lua. L'éditeur et le script agissent sur la même scène et peuvent être utilisés ensemble : une partie d'une animation peut être écrite en script puis ajustée dans l'éditeur, et inversement.

Le projet s'adresse aux enseignants, aux étudiants et à toute personne qui explique des mathématiques, sans exiger de compétences en programmation.

## Fonctionnalités prévues

- **Éditeur visuel** : outils de construction (points, segments, cercles, intersections), inspecteur de propriétés, timeline par étapes et aperçu en direct.
- **Géométrie dynamique** : les objets dépendent les uns des autres et la figure se met à jour quand un élément change.
- **Scripts Lua** : tout ce qui est faisable dans l'éditeur l'est aussi en script.
- **Formules mathématiques** : composition avec [Typst](https://typst.app), intégré à l'application.
- **Animations** : tracé progressif, fondus, transformations d'un objet en un autre, écriture de formules, mise en évidence.
- **Export vidéo** au format MP4 et mode présentation étape par étape.
- **3D** (à plus long terme) : courbes, surfaces et sections coniques dans l'espace.

## Architecture

Une scène est une donnée enregistrée dans un fichier `.zl`. Chaque image est calculée par une fonction pure du temps, ce qui permet de se placer à n'importe quel instant de la timeline. L'aperçu et l'export vidéo utilisent le même chemin de rendu.

```mermaid
flowchart TD
    E["Éditeur"] --> C["Commandes"]
    S["Script Lua"] --> C
    C --> SC["Scène<br/>objets, dépendances, timeline"]
    SC <--> F["Fichier .zl"]
    SC --> EV["Évaluation à l'instant t"]
    EV --> P["Primitives 2D"]
    P --> R["Rendu"]
    R --> A["Aperçu en direct"]
    R --> X["Export MP4"]
```

La racine du dépôt est le paquet `zetaanim`, qui produit l'exécutable de l'application et sert aussi de racine au workspace Cargo. Les bibliothèques sont dans `crates/` et sont ajoutées au fil du développement :

| Crate               | Rôle                                                                                  |
| ------------------- | ------------------------------------------------------------------------------------- |
| `zetaanim` (racine) | Exécutable de l'application, qui lance l'éditeur                                      |
| `zl-math`           | Interpolation, courbes d'animation, courbes de Bézier, géométrie, fonctions spéciales |
| `zl-scene`          | Modèle de scène : objets, paramètres, dépendances, timeline, animations, commandes    |
| `zl-render`         | Rendu 2D avec [tiny-skia](https://github.com/linebender/tiny-skia)                    |
| `zl-tex`            | Composition des formules avec Typst                                                   |
| `zl-export`         | Export vidéo via ffmpeg                                                               |
| `zl-script`         | API de script Lua                                                                     |
| `zl-editor`         | Interface de l'éditeur, construite avec [egui](https://github.com/emilk/egui)         |

## Feuille de route

- [ ] Mise en place du dépôt et de l'intégration continue
- [ ] Moteur sans interface et export MP4
- [ ] Éditeur minimal avec aperçu en direct
- [ ] Formules mathématiques
- [ ] Graphes de fonctions, constructions géométriques et transformations entre objets
- [ ] Scripts Lua et intégration avec l'éditeur
- [ ] Première version publique : exécutables pour macOS et Windows, galerie d'exemples
- [ ] Couche 3D

## Compilation depuis les sources

Prérequis :

- [Rust](https://www.rust-lang.org/tools/install) 1.85 ou plus récent (édition 2024)
- [ffmpeg](https://ffmpeg.org), pour l'export vidéo

```sh
git clone <url-du-dépôt>
cd zetaanim
cargo run --release
```

Aucun exécutable n'est encore fourni à ce stade.

## Contribuer

Les retours, les rapports de bugs et les propositions sont les bienvenus via les issues. Un guide de contribution sera publié avec les premières versions fonctionnelles.

## Licence

ZetaAnim est distribué au choix sous l'une des licences suivantes :

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) ou <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) ou <https://opensource.org/licenses/MIT>)

Sauf mention contraire explicite, toute contribution soumise pour inclusion dans le projet, telle que définie dans la licence Apache-2.0, est distribuée sous cette double licence, sans condition supplémentaire.
