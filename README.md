# Projet de Carte Aléatoire avec Robots Explorateurs et Extracteurs

## Description

Ce projet est une simulation d'une carte 2D aléatoire où des robots explorateurs et extracteurs naviguent pour découvrir et collecter des ressources (énergie et cristaux). Les robots explorateurs parcourent la carte pour trouver des ressources, puis retournent à la base pour laisser place à des robots extracteurs et rapporter les ressources à la base. La carte est générée aléatoirement avec des obstacles et une base. Les robots explorent la carte en dévoilant le brouillard de guerre autour d'eux.

## Prérequis

Pour exécuter ce projet, vous aurez besoin de :

- **Rust et Cargo** : Assurez-vous d'avoir Rust et Cargo installés sur votre machine. Vous pouvez les installer à partir de [rust-lang.org](https://www.rust-lang.org/).
- **Bibliothèque ggez** : ggez est une bibliothèque de jeu en Rust. Vous pouvez l'installer en utilisant Cargo.

## Installation

### Installer Rust et Cargo

Suivez les instructions sur [rust-lang.org](https://www.rust-lang.org/) pour installer Rust et Cargo.

### Créer un nouveau projet Cargo

Ouvrez votre terminal et créez un nouveau projet :

```
cargo new nom_du_projet
cd nom_du_projet
```
`### Ajouter ggez comme dépendance
Ouvrez le fichier Cargo.toml dans votre projet et ajoutez ggez, noise et rand à la section [dependencies] :

```
[dependencies]
ggez = "0.6.0-rc.1"
noise = "0.7.0"
rand = "0.8.4"
```
## Lancement du Projet
### Cloner le projet
Clonez ce dépôt dans votre répertoire local :

```
git clone https://github.com/votre-utilisateur/nom_du_projet.git
cd nom_du_projet
```
## Compiler et exécuter le projet
### Compilez et exécutez le projet en utilisant Cargo :

```
cargo run
```
## Fonctionnalités
Carte Aléatoire : La carte est générée de manière aléatoire avec des obstacles et une base.
Exploration et Extraction : Les robots explorateurs trouvent des ressources et les robots extracteurs les ramènent à la base.
Brouillard de Guerre : Les robots dévoilent le brouillard de guerre autour d'eux en explorant la carte.
Scores : Les scores pour les cristaux et l'énergie collectés sont affichés à l'écran.
## Auteur
Projet créé par Valentin Roche, Hubert Truang, Archibald Sabatier, Milo Roche et Nicolas Floris.
