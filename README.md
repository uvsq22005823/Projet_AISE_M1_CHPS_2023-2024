Ce projet a été écrit en Rust et peut être compilé en utilisant Cargo.

# À propos #

Le projet a été fait en Rust dans le but de faciliter l'usage des threads, le parsing des entrées et la gestion des erreurs. (et parce que j'aime bien le Rust)

Il reste des choses à implémenter : 
  - Meilleure robustesse pour le stockage des données
  - Extinction propre des threads (le programme ne s'arrête actuellement que via SIGINT SIGKILL...)
  - Gestion par l'utilisateur du nombre de threads max, fichier de stockage etc
  - Plus de commandes et types de Redis

# Fonctionalités #

Compatibilité partielle avec le protocole Redis :
  - Commandes PING, SET, GET, DEL, SAVE (partiel)
  - Multi-threading (1 client / thread)
  - Accès concurrents aux données
  - Persistance des données
  - Gestion de certains types : (Simple strings, Simple Errors, Integers, Bulk strings, Arrays, Nulls, Booleans)

  Compatibilité de la commande PING sans argument pour la commande inline
    => possibilité d'utiliser le test ping de redis-benchmark

### PING ###

  La commande PING renvoie les arguments passés, ou PONG si il n'y en a pas.

### SET ###
  La commande SET supplante la clef passée en argument avec la valeur fournie.

### GET ###
  La commande GET récupère les valeurs associées aux clefs passées en argument, et renvoie un type Null le cas échéant.

### DEL ###
  La commande DEL supprime les entrées passées en argument.

### SAVE ###
  La commande SAVE sauvegarde les données sur le disque. Contrairement à celle de Redis, cela ne fait que sauvegarder les données.

## Multi-threading ##
  Il y a un nombre maximum de threads à la fois, modifiable dans le code à la ligne 153 de main.rs (je n'ai pas eu le temps de gérer le passage d'arguments au programme, navré).
  Par défaut, il peut y en avoir 500.

## Persistance des données ##
  Les données sont par défaut stockées dans le fichier stockage.json, mais cela peut être modifié à la ligne 122 de main.rs
  Les données sont sauvegardées automatiquement toutes les 5 minutes (modifiable ligne 80 de main.rs) mais peuvent être sauvegardées manuellement avec la commande SAVE.
