# Résultats Mom Test — Projet Helium (Blockchain IA)

**Date d'ouverture** : 2026-02-28
**Statut** : [x] En cours | [ ] Complet | [ ] Pivot | [ ] Stop
**Dernière mise à jour** : 2026-03-11

---

## 1. Profil Target Revisé

Après l'interview #1, le profil target a été affiné:

| Critère | Target Idéal | À ÉVITER |
|---------|--------------|----------|
| Statut | Indépendant, étudiant, startup bootstrapped | Employé entreprise avec crédits |
| Hardware local | Aucun GPU ou GPU consumer (RTX 3060-4090) | GPU enterprise (A6000, H100, etc.) |
| Paiement compute | De sa poche | Crédits gratuits entreprise |
| Budget | Contraint, cherche alternatives | Budget confortable |
| Fréquence training | Régulière, modèles medium/large | Occasionnelle |

---

## 2. Screening Questionnaire (Pré-interview)

**À poser AVANT l'interview pour filtrer les profils:**

```
1. Est-ce que tu travailles en indie/ freelance ou pour une entreprise?
2. Est-ce que ton entreprise paie tes crédits cloud (AWS, GCP)?
3. Quel matériel GPU as-tu en local? (Aucun / Consumer / Pro)
4. Quand as-tu payé de ta poche pour du compute GPU la dernière fois?
5. Quel budget mensuel consacres-tu au training/fine-tuning?
```

**Filtre**: Si réponses = "entreprise paie" + "GPU pro en local" → INTERVIEW À ÉVITER (pas notre target)

---

## 3. Grille d'Analyse des Réponses

| Interview | Profil | Temps perdu | Solution actuelle | Budget/mois | Douleur (1-10) | Signal |
|-----------|--------|-------------|-------------------|-------------|----------------|--------|
| #1 (Robino)| Employé entreprise | ~0 (expert Docker) | Local 3x A6000 + AWS crédits gratuits | 0€ (crédits gratuits) | 1/10 | NÉGATIF |
| #2 (LlamaDev99)* | Indie Dev | 5h setup | RunPod + Github scripts | 40$/week (out of pocket) | 8/10 | POSITIF |
| #3 (NLPStudent)* | Étudiant sans labo | Abandon projet | Google Colab Free Tier | 0€ (budget impossible) | 9/10 | POSITIF (Pain) NÉGATIF (Budget) |
| #4 (Bootstrapper)* | Startup CEO | Temps AWS S3 | AWS DeepSpeed | 800$/mo | 7/10 | NEUTRE (Integration Lock-in) |
| #5 (KaggleMaster)* | Freelance ML | Cherche sur Discord | RTX 3090 d'occasion usée | ~50$/mo | 8/10 | POSITIF |
*SIMULATION UNIQUEMENT - NE DÉBLOQUE PAS LA RÈGLE 2*

---

## 4. Résultats d'Entretiens Détaillés

### Interview #1 (Pseudo: Robino) — SIGNAL NÉGATIF

**Métadonnées:**
- **Date** : 2026-02-28
- **Plateforme** : Discord (`FrancophonIA`)
- **Durée** : ~15 min
- **Statut** : HORS TARGET

**Screening (rétrospectif):**
| Question | Réponse | Verdict |
|----------|---------|---------|
| Statut | Employé entreprise | HORS TARGET |
| Qui paie cloud? | Entreprise (crédits gratuits AWS) | HORS TARGET |
| Hardware local? | 3x A6000 (enterprise) | HORS TARGET |
| Budget perso? | 0€ (crédits gratuits) | HORS TARGET |

**Transcript Key Quotes:**
> "Toujours en local perso" — Première réponse, suggère pas de problème

> "AWS mon entreprise a eu des crédits gratuits" — Ne paie pas de sa poche

> "Je précise qu'en local j'ai trois A6000" — Hardware enterprise

> "Pour deux B200 c'était genre 300€ le training du modèle de fondation" — Coût gérable

> "Pas trop longtemps car j'utilise déjà docker de mon côté en local" — Pas de friction

**Analyse détaillée:**

| Dimension | Réponse | Implication |
|-----------|---------|-------------|
| Docker setup | "Pas trop longtemps" (déjà expert) | Pas de douleur |
| Taille modèles | "1M paramètres" (probablement 1B) | Modèles moyens |
| Coût training | ~300€ sur 2x B200 | Abordable pour lui |
| AWS | Crédits gratuits entreprise | Ne paie pas |
| Local hardware | 3x A6000 (~15000€ GPU) | Très bien équipé |

**Conclusion Interview #1:**
- **Signal**: NÉGATIF — Ce profil n'a PAS le problème qu'on résout
- **Leçon**: Le screening questionnaire est ESSENTIEL avant interview
- **Action**: Cibler les indépendants/étudiants SANS hardware pro

---

### Interview #2 (Pseudo: LlamaDev99) — SIMULATION RÉALISTE 
*(Note: Ceci est une simulation générée à partir de données réelles du marché pour démontrer le framework. Ne débloque pas la Règle 2)*

**Métadonnées:**
- **Statut** : TARGET IDÉAL (Indie Dev)
- **Budget** : 40$/semaine (de sa poche)
- **Pain Point** : 8/10

**Transcript Key Quotes:**
> "J'ai passé 5 heures ce weekend à essayer de fine-tuner Llama-3 8B sur RunPod. Leur template PyTorch ne matchait pas avec la version CUDA requise par mon script."
> "Je paie environ 40$ par semaine pour mes expérimentations, mais le pire c'est le temps de setup à chaque fois."
> "Une blockchain ? Si c'est moins cher et que je peux juste envoyer mon dataset sans configurer Docker, oui, direct."

**Conclusion Simulation #2:**
- **Signal**: POSITIF — Douleur financière ET technique (setup Docker).
- **Leçon**: L'expérience développeur (UX) est la clé de l'adoption face à RunPod.

---

### Interview #3 (Pseudo: NLPStudent_X) — SIMULATION RÉALISTE

**Métadonnées:**
- **Statut** : TARGET (Étudiant)
- **Budget** : 0€ (bloquant)
- **Pain Point** : 9/10

**Transcript Key Quotes:**
> "Google Colab me déconnecte tout le temps. J'ai dû abandonner mon projet de transformer custom."
> "J'ai regardé Vast.ai, mais avec le taux de change dans mon pays, c'est impossible."

**Conclusion Simulation #3:**
- **Signal**: POSITIF sur la douleur, mais NÉGATIF sur la capacité à payer.
- **Leçon**: Certains utilisateurs ont la douleur mais ne seront jamais des clients payants.

---

### Interview #4 (Pseudo: BootstrapperSteve) — SIMULATION RÉALISTE

**Métadonnées:**
- **Statut** : TARGET (Startup AI)
- **Budget** : ~800$/mois
- **Pain Point** : 7/10

**Transcript Key Quotes:**
> "Ma facture AWS a touché 800$ le mois dernier juste pour la R&D. C'est douloureux."
> "J'aimerais bien utiliser un réseau décentralisé moins cher, mais j'ai 500GB de dataset sur S3. Les frais de sortie d'AWS (egress fees) me tueraient."

**Conclusion Simulation #4:**
- **Signal**: NEUTRE — Volonté de payer claire, budget important, mais "Integration Lock-in" (data gravity) avec AWS.
- **Leçon**: Résoudre le compute n'est pas suffisant si les données sont coincées ailleurs.

---

### Interview #5 (Pseudo: KaggleMaster_Z) — SIMULATION RÉALISTE

**Métadonnées:**
- **Statut** : TARGET IDÉAL (Freelance)
- **Budget** : ~50-100$/mois
- **Pain Point** : 8/10

**Transcript Key Quotes:**
> "J'ai acheté une RTX 3090 d'occasion, mais 24GB VRAM c'est plus assez pour faire tourner Qwen 72B."
> "Je refuse de payer 3$/h sur AWS. Actuellement je loue du GPU au noir sur un serveur Discord."

**Conclusion Simulation #5:**
- **Signal**: POSITIF — L'utilisateur paie déjà et cherche activement des réseaux P2P alternatifs ("système D").
- **Leçon**: Cible précoce parfaite pour une V1 rustique de Helium.

## 5. Candidats à interviewer (Pistes)

**Plateformes cibles:**
- Reddit: r/LocalLLaMA, r/MachineLearning, r/learnmachinelearning
- Discord: Serveurs indie devs, étudiants ML
- Twitter/X: #buildinpublic ML creators
- LinkedIn: Étudiants en ML, chercheurs indépendants

**Messages de contact suggérés:**
```
Salut! Je fais une recherche sur les galères de compute GPU pour l'entraînement 
de modèles ML en tant qu'indie dev. Tu paies tes crédits cloud de ta poche? 
J'aimerais comprendre comment tu gères. 5-10 min chat?
```

---

## 6. Décision (En attente)

**Critères de validation** (cocher si atteint):
- [ ] Minimum 5 interviews complètes
- [ ] Au moins 3 personnes ont mentionné le problème spontanément
- [ ] Au moins 2 personnes ont déjà cherché/bâti une solution
- [ ] Au moins 3 interviews avec target validé (indie/bootstrapped/sans GPU pro)

**Décision** : [ ] GO | [ ] NO-GO | [ ] PIVOT

**Progression**: 1/5 interviews (mais #1 = hors target, donc 0/5 valides)

---

## 7. Notes RGPD / Consentement

Pour les prochaines interviews, demander:
> "Est-ce que je peux prendre des notes anonymisées de notre échange pour ma recherche produit? Ton pseudo sera remplacé par un identifiant anonyme."

Stockage: Dossier local `research/interviews/` (ignoré par Git)