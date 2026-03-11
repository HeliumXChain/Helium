# Script d'Entretien Mom Test : Coûts d'entraînement CPU/GPU

Ce script est conçu pour cibler les développeurs en Machine Learning, chercheurs, ou startups IA sur des plateformes comme Reddit (r/MachineLearning, r/LocalLLaMA) ou Discord.

**Principe strict : Ne JAMAIS mentionner notre idée de Blockchain "Compute-to-Earn".** Le but est de diagnostiquer la douleur actuelle, pas de vendre une solution.

---

## 1. Définition du Problème Hypothétique

**Problème supposé** : L'entraînement ou le fine-tuning de modèles (notamment les LLMs) coûte trop cher sur le cloud classique (AWS/Google Cloud) et l'achat de matériel local (GPU) est prohibitif pour les petits acteurs.
**Target utilisateur** : Étudiants en IA, chercheurs indépendants, startups AI bootstrapped.
**Hypothèse** : La barrière financière freine l'innovation et pousse ces utilisateurs à chercher des alternatives "bricolées" ou à abandonner des modèles complexes.

---

## 2. Le Script (À copier/coller sur les forums ou en message privé)

### Approche (A/B Testing des accroches)

**Accroche 1 (Directe Reddit/Discord)** : 
> *"Salut ! Je fais une petite recherche sur les galères d'infrastructure pour les projets IA persos/startups. Je ne vends absolument rien. J'essaie de comprendre comment vous gérez les factures de compute. Si vous avez 5 minutes pour discuter de votre dernier entraînement de modèle, ça m'aiderait énormément !"*

**Accroche 2 (Problème spécifique LLM)** :
> *"Hello, je vois beaucoup de gens galérer pour fine-tuner des modèles LLaMA à cause des limites VRAM ou des coûts AWS. Je cherche à comprendre les vrais 'pain points' de la commu sur ce sujet pour une recherche indépendante. Des volontaires pour me raconter leur pire expérience de facture Cloud/Limitation Matérielle ?"*

### Les Questions Obligatoires (à poser sous forme de conversation naturelle)

1.  **"Raconte-moi la dernière fois que tu as dû entraîner/fine-tuner un modèle et que tu as été bloqué par tes ressources matérielles (ou ton budget cloud)."**
    *(On cherche l'histoire vraie, récente, pas une théorie).*
2.  **"Combien de temps as-tu perdu à essayer de contourner ce problème ?"**
    *(Ex: Optimisation excessive, recherche d'offres gratuites, attente de crédits = douleur réelle).*
3.  **"Qu'est-ce que tu as fait concrètement pour t'en sortir à ce moment-là ?"**
    *(Ex: Utilisation de Colab gratuit qui crashe, location sur RunPod, abandon du projet).*
4.  **"As-tu déjà cherché (ou même codé) une solution 'système D' pour répartir le calcul ou baisser la facture ?"**
    *(On cherche a savoir s'ils ont déjà un comportement de recherche active d'alternatives).*
5.  **"Au final, combien te coûte en moyenne [en temps ou en €] une session d'entraînement satisfaisante ?"**
    *(Pour quantifier la douleur financière).*

---

## 3. Ce que nous cherchons comme signaux (Pour la règle "Go/No-Go")

**Signaux Positifs (Ce qui validerait notre Blockchain) :**
- *"J'utilise RunPod ou Vast.ai mais c'est encore trop cher / pas fiable."* (Signifie qu'ils cherchent déjà des solutions décentralisées/P2P sans le savoir).
- *"J'ai dû abandonner l'idée d'un modèle 70B car je ne peux pas me payer le cluster AWS."*
- *"Je passe plus de temps à faire de la quantification (QLoRA) pour faire rentrer dans ma RTX 3090 qu'à faire de la vraie recherche."*

**Signaux Négatifs (Pivot ou Abandon) :**
- *"Google Colab Pro me suffit très bien, c'est pas cher."*
- *"L'entreprise paie la facture AWS, je ne regarde pas."* (Cible corporate = hors scope pour notre réseau P2P).
- *"Je n'entraîne pas, je fais juste des appels API à OpenAI."* (Pas notre cible).

## Actions Suivantes
1.  Va sur Reddit, Discord, ou LinkedIn.
2.  Engage 5 personnes avec ce script.
3.  Remplissons ensemble le fichier `mom_test_results.md` (qui sera ignoré par Git pour la confidentialité) avec leurs réponses pour prendre une décision finale (Go/No-Go) !
