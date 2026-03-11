Quick questions avant de commencer:
1. Tu travailles en indie/freelance ou pour une entreprise?
2. Est-ce que ton entreprise paie tes crédits cloud (AWS, GCP)?
3. Quel matériel GPU as-tu en local? (Aucun / RTX 3060-4090 / A6000+)
4. Tu paies tes crédits compute de ta poche?
5. C'est quoi ton plus gros training/fine-tuning récent?
```

**Filtre de validation:**
| Réponse | Verdict |
|---------|---------|
| Entreprise + crédits gratuits + GPU pro | REJETER - Hors target |
| Indie/freelance + paie de sa poche + pas de GPU pro | ACCEPTER - Target idéal |
| Étudiant + budget limité + pas de GPU | ACCEPTER - Target idéal |
| Bootstrapped + budget serré | ACCEPTER - Target idéal |

---

## 1. Définition du Problème Hypothétique

**Problème supposé** : L'entraînement ou le fine-tuning de modèles (notamment les LLMs) coûte trop cher sur le cloud classique (AWS/Google Cloud) et l'achat de matériel local (GPU) est prohibitif pour les petits acteurs.

**Target utilisateur RÉVISÉ** : 
- Étudiants en IA SANS labo équipé
- Chercheurs indépendants SANS budget
- Startups AI bootstrapped SANS credits gratuits
- Indie devs SANS GPU pro

**Hypothesis** : La barrière financière freine l'innovation et pousse ces utilisateurs à chercher des alternatives "bricolées" ou à abandonner des modèles complexes.

**Anti-target (À ÉVITER)** :
- Employés avec credits entreprise
- Users avec A6000/H100 en local
- Ceux qui paient pas de leur poche

---

## 2. Le Script (À copier/coller sur les forums ou en message privé)

### Approche (A/B Testing des accroches)

**Accroche 1 (Reddit r/LocalLLaMA, r/MachineLearning)** : 
> *"Salut ! Je suis indie dev et je galère avec les coûts GPU pour mes projets ML. Je fais une recherche pour comprendre comment les autres gèrent. Si tu paies tes crédits cloud de ta poche, j'aimerais chatter 5 min. Pas de pitch, juste comprendre ton expérience."*

**Accroche 2 (Discord serveurs indie/étudiants)** :
> *"Hello, je cherche des devs qui payent leur compute GPU de leur poche pour une recherche produit. Si tu as déjà abandonné un projet à cause du coût GPU, je veux comprendre. 5-10 min max."*

**Accroche 3 (Twitter/X #buildinpublic)** :
> *"Indie ML devs: ever abandoned a project because GPU costs were too high? I'm researching this problem. DM me if you pay compute from your own pocket. Just want to understand your pain - no pitch."*

### Les Questions Obligatoires (à poser sous forme de conversation naturelle)

**PHASE 1: Screening (valider le profil)**
1. *"Tu travailles en solo/indie ou pour une boite?"*
2. *"Qui paie tes crédits cloud - toi ou l'entreprise?"*
3. *"Tu as quoi comme GPU en local?"*

**PHASE 2: Histoire de douleur (si profil validé)**
4. **"Raconte-moi la dernière fois que tu as dû entraîner/fine-tuner un modèle et que tu as été bloqué par tes ressources (ou ton budget)."**
   *(On cherche l'histoire vraie, récente, pas une théorie).*
   
5. **"Combien de temps as-tu perdu à essayer de contourner ce problème?"**
   *(Ex: Optimisation excessive, recherche d'offres gratuites, attente de crédits = douleur réelle).*
   
6. **"Qu'est-ce que tu as fait concrètement pour t'en sortir à ce moment-là?"**
   *(Ex: Utilisation de Colab gratuit qui crashe, location sur RunPod, abandon du projet).*

7. **"Combien ça t'a coûté au final? (en temps ou en euros)"**
   *(Pour quantifier la douleur financière).*

**PHASE 3: Comportement de recherche**
8. **"As-tu déjà cherché (ou même codé) une solution 'système D' pour répartir le calcul ou baisser la facture?"**
   *(On cherche a savoir s'ils ont déjà un comportement de recherche active d'alternatives).*

9. **"Si tu avais accès à du GPU moins cher mais décentralisé (genre location P2P), tu serais prêt à l'utiliser?"**
   *(Attention: ne pas pitcher Helium, juste sonder l'ouverture).*

---

## 3. Ce que nous cherchons comme signaux (Pour la règle "Go/No-Go")

**Signaux Positifs (Ce qui validerait notre Blockchain):**
- *"J'ai dû abandonner le projet car Colab free crashait tout le temps."*
- *"Je passe plus de temps à faire de la quantization pour faire rentrer dans ma RTX 3060 qu'à faire de la vraie recherche."*
- *"J'ai essayé RunPod, Vast.ai, Lambda Labs... c'est toujours trop cher pour moi."*
- *"Je paie ~200-500€/mois de compute de ma poche, c'est pas viable."*
- *"J'ai dû réduire la taille de mon modèle à cause du budget."*

**Signaux Négatifs (Pivot ou Abandon):**
- *"Mon entreprise paie tout, je ne regarde pas la facture."* (Hors target)
- *"J'ai un cluster de 4x A100 au travail."* (Hors target)
- *"Google Colab Pro me suffit très bien, c'est pas cher."* (Pas de douleur)
- *"Je n'entraîne pas, je fais juste des appels API à OpenAI."* (Pas notre cible)

**Signaux Neutres (à approfondir):**
- *"J'utilise Paperspace/RunPod mais c'est correct."* (Douleur faible mais existe)

---

## 4. Plateformes cibles

| Plateforme | Canaux | Type de user |
|------------|--------|--------------|
| Reddit | r/LocalLLaMA, r/MachineLearning, r/learnmachinelearning | Mixte |
| Discord | Serveurs indie devs, ML students, EleutherAI | Technique |
| Twitter/X | #buildinpublic, #indiedev, #machinelearning | Indies |
| LinkedIn | Étudiants ML, chercheurs indépendants | Pro/académique |
| HuggingFace | Forums, Discord | ML practitioners |

---

## 5. Consentement RGPD

**À demander avant de noter:**
> *"Est-ce que je peux prendre des notes anonymisées de notre échange pour ma recherche produit? Ton pseudo sera remplacé par un identifiant anonyme et les notes resteront privées."*

**Stockage:** Dossier local `research/interviews/` (ignoré par Git)

---

## 6. Actions Suivantes

1. Identifier 10 candidats potentiels sur les plateformes cibles
2. Envoyer les messages d'approche (accroches 1, 2 ou 3)
3. Filtrer avec le screening questionnaire
4. Mener 5 interviews valides (profil target confirmé)
5. Remplir `mom_test_results.md` avec les résultats
6. Prendre la décision GO/NO-GO/PIVOT

**Objectif**: 5 interviews valides avec target confirmé = Mom Test complet