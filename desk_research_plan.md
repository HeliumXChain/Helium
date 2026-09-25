# Desk Research Plan — Helium Communities Validation

**Objectif** : Valider le besoin par recherche secondaire (preuves existantes)
**Methode** : Analyse de concurrents, discussions en ligne, donnees de marche
**Duree** : 2-4 heures
**Livrable** : Rapport avec evidences documentees

---

## 1. Concurrents et Solutions Similaires

### Recherche a effectuer

**Direct competitors** (P2P compute sharing):
- [ ] Bittensor (TAO) — Decentralized AI compute
- [ ] Akash Network — Decentralized cloud computing
- [ ] Golem Network — P2P computing marketplace
- [ ] iExec — Decentralized marketplace for computing
- [ ] Render Network — GPU rendering marketplace
- [ ] Salad.com — GPU sharing for gamers
- [ ] Vast.ai — GPU marketplace

**Indirect competitors** (solutions traditionnelles):
- [ ] RunPod
- [ ] Lambda Labs
- [ ] CoreWeave
- [ ] Google Colab Pro
- [ ] Paperspace
- [ ] AWS EC2 GPU instances

**Niches similaires**:
- [ ] Folding@home (volunteer computing)
- [ ] SETI@home (historical reference)
- [ ] BOINC projects

### Questions a repondre

1. Quels sont les modeles economiques existants ?
2. Quels sont les points de friction des solutions actuelles ?
3. Y a-t-il des cas de P2P "entre amis" deja documentes ?
4. Quels sont les taux d'adoption et de retention des plateformes P2P ?

### Sources
- Sites officiels des concurrents
- Documentation technique
- Whitepapers
- Blog posts "Why we built X"
- Reddit discussions sur ces plateformes

---

## 2. Discussions et Demandes en Ligne

### Subreddits a analyser

**Communautes ML/AI**:
- [ ] r/LocalLLaMA — Discussions sur fine-tuning local
- [ ] r/MachineLearning — General ML compute needs
- [ ] r/learnmachinelearning — Beginners with limited resources
- [ ] r/deeplearning — Deep learning compute
- [ ] r/selfhosted — Self-hosted infrastructure
- [ ] r/homelab — Home lab enthusiasts with hardware

**Communautes dev**:
- [ ] r/programming
- [ ] r/coding
- [ ] r/startups — Resource constraints

**Questions cles a chercher**:
- "How to fine-tune without expensive GPU"
- "Alternative to Colab for training"
- "Share GPU between friends"
- "Cheap GPU for machine learning"
- "Run LLM on low RAM"
- "Borrow GPU compute"

### Discord a explorer

- [ ] FrancophonIA — Communaute francophone ML
- [ ] EleutherAI — Open source AI research
- [ ] LAION — Open source datasets/models
- [ ] Various ML Discords (requires search)

### Twitter/X a scraper

**Hashtags et keywords**:
- #llm #training #gpu #compute #aws #colab #machinelearning
- "can't afford AWS GPU"
- "Colab keeps disconnecting"
- "need GPU for fine-tuning"

### Forums et autres

- [ ] Hacker News — "Ask HN: How do you train models without cloud costs?"
- [ ] Stack Overflow — GPU sharing questions
- [ ] GitHub Discussions — LLM fine-tuning repos

---

## 3. Donnees de Marche

### Marche du compute AI

**A rechercher**:
- [ ] Taille du marche GPU cloud computing (2024-2026)
- [ ] Cout moyen de training pour LLMs
- [ ] Pourcentage de devs qui paient pour du compute cloud
- [ ] Statistiques sur Google Colab usage
- [ ] Adoption des solutions decentralisees (Akash, Golem)

**Segments utilisateurs**:
- [ ] Nombre de ML engineers independants
- [ ] Etudiants en ML/Data Science
- [ ] Startups AI bootstrapped
- [ ] Researchers academiques avec budget limite

### Pain points documentes

**A rechercher**:
- [ ] "Barriers to entry" ML training
- [ ] "Cost of AI training" surveys
- [ ] "Developer pain points" compute
- [ ] Academic papers sur l'acces democratique au compute

### Sources
- [ ] Gartner / IDC reports (AI infrastructure)
- [ ] Papers with Code — compute trends
- [ ] Hugging Face surveys
- [ ] State of AI reports
- [ ] Developpeur ML surveys

---

## 4. Validation Comportementale (Observed Behavior)

### Signaux de validation

**Comportements existants**:
- [ ] Des devs partagent deja leurs serveurs entre eux ? (preuves ?)
- [ ] Des communautes de "GPU sharing" informelles ?
- [ ] Des demandes de "borrow GPU" sur Reddit/forums ?
- [ ] Des projets open-source de P2P compute pour ML ?

**Failed attempts**:
- [ ] Des projets similaires qui ont echoue et pourquoi ?
- [ ] Des pivots de projets P2P compute ?

---

## 5. Rapport de Synthese

### Structure du rapport

1. **Executive Summary** (1 page)
   - Conclusion principale : Le besoin existe / n'existe pas
   - Niveau de confiance (High/Medium/Low)
   - Recommandation : GO / NO-GO / PIVOT

2. **Concurrent Analysis** (2-3 pages)
   - Tableau comparatif des solutions existantes
   - Gaps identifies
   - Opportunites de differenciation

3. **Community Evidence** (2-3 pages)
   - Screenshots des discussions pertinentes
   - Count des "mentions spontanees"
   - Profils des utilisateurs qui se plaignent

4. **Market Data** (1-2 pages)
   - TAM/SAM/SOM
   - Statistiques cles
   - Trends

5. **Recommendation** (1 page)
   - GO — Probleme valide, lancer MVP
   - NO-GO — Pas de besoin documente
   - PIVOT — Ajuster l'approche

---

## Checklist de Validation

### Minimum pour "GO"

- [ ] 3+ concurrents similaires (preuve que le probleme est reconnu)
- [ ] 10+ discussions Reddit avec ce probleme (preuve de demande)
- [ ] 1+ solution P2P similaire qui existe deja (preuve de marché)
- [ ] Marche > $100M (preuve de scalabilite)

### Red flags pour "NO-GO"

- [ ] 0 concurrents (personne n'a essaye)
- [ ] Toutes les discussions sont "je voudrais si c'etait gratuit"
- [ ] Solutions existantes avec 0 traction
- [ ] Probleme deja resolu par une solution gratuite

---

## Tools pour la recherche

### Recherche web
- Google (filtre "Past year" pour actualite)
- Google Scholar (papers academiques)
- Reddit search (site:reddit.com)
- Twitter Advanced Search

### Outils d'analyse
- Notion / Obsidian — Notes
- Excel / Sheets — Tableaux comparatifs
- Screenshot tool — Preuves visuelles
- Wayback Machine — Historique des projets

---

## Timeline

| Heure | Tache |
|-------|-------|
| 0:00-0:30 | Setup et recherche concurrents |
| 0:30-1:30 | Reddit/Discord research |
| 1:30-2:30 | Market data research |
| 2:30-3:30 | Synthese et rapport |
| 3:30-4:00 | Decision GO/NO-GO/PIVOT |

---

## Prochaines etapes apres desk research

Si **GO** :
1. Lancer MVP (2 transactions)
2. Creer landing page avec v0
3. Poster sur X

Si **NO-GO** :
1. Documenter pourquoi
2. Pivot sur Helium Core (blockchain PoUW)
3. Reconsiderer le probleme

Si **PIVOT** :
1. Ajuster l'hypothese
2. Nouveau desk research
3. Nouveau plan validation
