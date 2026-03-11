# Proposition de Développement pour le Projet Helium

Le projet "Helium" a pour objectif la construction d'une blockchain indépendante s'appuyant sur le protocole Tor. L'état actuel du code consiste en une ébauche initiale de la classe `Blockchain` dans `test_blockchain.py`. 

Afin de structurer le développement de manière rigoureuse et scientifique, les axes suivants sont proposés :

## 1. Architecture Réseau et Intégration Tor
- **Topologie P2P** : Implémentation de la communication entre les nœuds (pairs) exclusivement via des sockets proxySocks (ex. via la bibliothèque `stem` ou `PySocks`) pour garantir l'anonymat.
- **Services Cachés (Hidden Services)** : Configuration des nœuds pour opérer en tant que services cachés Tor (`.onion`), permettant une localisation décentralisée sans exposition des adresses IP publiques.
- **Découverte des Pairs** : Conception d'un mécanisme d'amorçage (bootstrapping) de nœuds initiaux (hardcodés) sur le réseau Tor pour initier la connectivité réseau.

## 2. Structure et Cœur de la Blockchain
- **Modèle de Données** : Définition stricte et typée des structures de données fondamentales :
  - `Bloc` (Index, Timestamp, Hash Précédent, Nonce, Arbre de Merkle des transactions).
  - `Transaction` (Entrées, Sorties, Montants, Scripts de déverrouillage/verrouillage).
- **Consensus** : Implémentation d'un algorithme de consensus adapté aux latences inhérentes à Tor. Une Preuve de Travail (Proof of Work) dynamique ou une Preuve d'Enjeu (Proof of Stake) doit être considérée.
- **Validation** : Remplacement du code actuel par un algorithme complet de vérification de la validité cryptographique de la chaîne.

## 3. Cryptographie et Modélisation des Transactions
- **Courbes Elliptiques** : Intégration de la norme `secp256k1` pour la génération des clés publiques/privées et des signatures numériques (ECDSA).
- **Modèle Comptable** : Choix et implémentation du modèle UTXO (Unspent Transaction Output) afin de minimiser le risque de double dépense et d'assurer une vérifiabilité mathématique asynchrone sécurisée.

## 4. Architecture Logique du Dépôt
Il est recommandé d'organiser le dépôt selon le paradigme de ségrégation des responsabilités :
- `helium/core/` : Logique de la blockchain (blocs, chaîne, transactions, consensus).
- `helium/network/` : Interface avec Tor, synchronisation des pairs, requêtes P2P.
- `helium/crypto/` : Fonctions de hachage, arbres de Merkle, signatures.
- `helium/wallet/` : Gestion des clés cryptographiques et création des transactions.
- `tests/` : Batterie de tests unitaires et fonctionnels (ex. via `pytest`).

## Outils à Mettre en Place pour l'Environnement de Développement
- Utilisation d'un gestionnaire d'environnement strict tel que `Poetry` pour isoler les dépendances (`PySocks`, `stem`, `cryptography`, `ecdsa`).
- Paramétrage de hooks Git (`pre-commit`) pour faire respecter le formatage strict du code, tel qu'exigé par les règles récemment synchronisées depuis `kuro-rules`.
