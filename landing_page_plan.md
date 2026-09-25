# Landing Page + Waitlist — Helium Communities

**Objectif** : Valider l'interet par 100+ inscriptions avec profils qualifies
**Equivalence Mom Test** : 3 interviews si 3-5% conversion atteinte
**Deadline** : 1 semaine pour atteindre le seuil

---

## Structure Landing Page

### 1. Hero Section

**Headline** : "Emprunte la RAM/GPU d'un dev de confiance"
**Subheadline** : "Fine-tune tes LLMs sans payer AWS. Partage tes ressources, gagne des credits."
**CTA** : "Rejoindre la beta" (formulaire email)

### 2. Problem Section

**Titre** : "Tu connais cette douleur ?"
**Points** :
- Google Colab te deconnecte apres 2h
- AWS coute 300$/mois pour un fine-tuning
- Ton laptop crash sur les modeles > 7B
- Tu connais un dev avec un PC plus puissant mais tu n'oses pas demander

### 3. Solution Section

**Titre** : "Helium Communities — Partage de confiance"
**Features** :
- Reseau ferme de devs verifies (Web of Trust)
- Tunnel chiffre WireGuard entre machines
- Execution dans microVM isolee (Firecracker)
- Credits locaux : prete aujourd'hui, emprunte demain

### 4. How It Works

**Borrower** (besoin de compute) :
1. Trouve un provider dans ta communaute
2. Connecte-toi via tunnel securise
3. Fine-tune ton modele dans VM isolee
4. Paie en credits ou cash

**Provider** (ressources disponibles) :
1. Declare tes ressources disponibles
2. Accepte les demandes de confiance
3. Gagne des credits pour plus tard
4. Garde le controle (kill switch VM)

### 5. Social Proof

**Testimonial** (placeholder pour apres MVP Technique) :
> "J'ai fine-tune Llama-3 8B en 2h sur une workstation a un pote. Zero config AWS."
> — Dev indie, Paris

### 6. FAQ

**Q** : C'est quoi la difference avec RunPod/AWS ?
**R** : RunPod = strangers + cher. Helium = amis de confiance + 50% moins cher.

**Q** : C'est securise ?
**R** : Tunnel WireGuard (chiffrement militaire) + VM isolee (pas d'acces a tes fichiers).

**Q** : C'est gratuit ?
**R** : Beta gratuite. Puis paiement entre membres (credits ou cash).

**Q** : Quand c'est disponible ?
**R** : Beta ouverte Q2 2026. Inscris-toi pour acces prioritaire.

### 7. Final CTA

**Titre** : "Rejoins la beta"
**Texte** : "100+ devs deja sur la liste. Acces limite."
**Form** : Email + Profil (Borrower / Provider / Les deux)

---

## Form Waitlist Qualifiee

### Champs obligatoires

| Champ | Type | Options |
|-------|------|---------|
| Email | email | - |
| Profil | select | Borrower (besoin GPU) / Provider (ai GPU) / Les deux |
| Localisation | text | Ville, Pays |
| Hardware actuel | select | < 16GB / 16-32GB / 32-64GB / 64GB+ / GPU RTX / GPU Pro |
| Besoin actuel | select | Fine-tuning / Inference / Training / Curiosite |
| Frequence | select | Quotidien / Hebdo / Mensuel / Occasionnel |
| Budget mensuel | select | 0€ / <20€ / 20-50€ / 50-100€ / 100€+ |
| Interet | radio | Tres interesse / Curieux / A voir |

### Champs optionnels

- GitHub/Twitter (pour verification)
- Message libre (pain points specifiques)
- Consentement newsletter

---

## Tech Stack Landing Page

**Option 1 — Simple (Recommande)** :
- HTML/CSS statique sur Netlify
- Formspree pour le formulaire
- Gratuit, rapide a deployer

**Option 2 — Scalable** :
- Next.js sur Vercel
- Airtable pour la waitlist
- Analytics (Plausible ou Google)

**Option 3 — No-Code** :
- Carrd.co (19$/an)
- Tally.so pour le form
- Zapier pour automations

---

## Plan de Lancement (7 jours)

### Jour 1 — Setup (Mardi)
- [ ] Creer landing page HTML
- [ ] Setup formulaire (Formspree/Tally)
- [ ] Deploy sur Netlify/Vercel
- [ ] Test conversion (email de test)

### Jour 2 — Contenu (Mercredi)
- [ ] Rediger copy (hero, problem, solution)
- [ ] Ajouter FAQ
- [ ] Optimiser mobile
- [ ] Setup analytics

### Jour 3 — Soft Launch (Jeudi)
- [ ] Poster sur 1 sous-reddit (r/LocalLLaMA)
- [ ] Poster sur 1 Discord (FrancophonIA)
- [ ] Mesurer CTR et conversion
- [ ] Ajuster copy si besoin

### Jour 4 — Iteration (Vendredi)
- [ ] Analyser premiers signups
- [ ] Qualifier les profils (borrower vs provider)
- [ ] A/B test headline si trafic faible
- [ ] Doubler down sur meilleur channel

### Jour 5 — Scale (Weekend)
- [ ] Poster sur 3+ subreddits
- [ ] DM 10 devs sur Twitter/X
- [ ] Lancer thread technique
- [ ] Repondre aux commentaires/questions

### Jour 6 — Analyse (Lundi)
- [ ] Compter total inscrits
- [ ] Calculer taux conversion
- [ ] Identifier top segments
- [ ] Qualifier leads chauds

### Jour 7 — Decision (Mardi)
- [ ] Atteint 100+ inscrits ?
- [ ] 3-5% conversion au moins ?
- [ ] Profils qualifies (pas juste curieux) ?
- [ ] Documenter dans validation_evidence.md

---

## Metriques a Tracker

| Metrique | Outil | Seuil validation |
|----------|-------|------------------|
| Visites uniques | Plausible/Google | 1000+ |
| Inscriptions | Formspree/Tally | 100+ |
| Taux conversion | Calcule | 3-5% |
| Borrowers | Form | 60+ |
| Providers | Form | 40+ |
| Profils avec budget >20€ | Form | 30+ |

---

## Message de Lancement (Template)

### Reddit r/LocalLLaMA

```
J'ai cree un outil pour emprunter de la RAM/GPU a d'autres devs

Titre : I built a tool to borrow RAM/GPU from other devs (without AWS)

Body :
Je train beaucoup de LLMs et j'en ai marre de payer AWS ou de galere avec Colab qui deconnecte.

J'ai commence a travailler sur Helium Communities — un reseau ferme ou tu peux emprunter/emprunter du compute a des devs de confiance.

Ca marche comment :
- Tu trouves un dev avec un PC plus puissant dans ta communaute
- Tunnel WireGuard chiffre entre vos machines
- Execution dans microVM isolee (pas d'acces a ses fichiers)
- Paiement en credits (prete aujourd'hui, emprunte demain)

C'est pas pour remplacer AWS, c'est pour les cas ou tu as besoin de 16-32GB de plus pour un fine-tuning rapide.

Je cherche des beta testers. Si tu es interesse : [LIEN]

Questions bienvenues.
```

### Twitter/X

```
Tired of paying $300/month to fine-tune LLMs on AWS?

I'm building Helium Communities — borrow GPU/RAM from devs you trust.

- No AWS bills
- No Colab timeouts
- Just peer-to-peer compute sharing

Join the beta: [LIEN]

#buildinpublic #llm #machinelearning
```

### Discord (FrancophonIA)

```
Salut ! Je teste un concept de partage de ressources compute entre devs.

Probleme : Colab deconnecte, AWS coute cher, et plein de devs ont des workstations qui tournent en idle.

Solution : Reseau ferme pour emprunter/emprunter du compute a des devs de confiance (WireGuard + microVM).

Je cherche des volontaires pour la beta. Ca prend 2 min de s'inscrire : [LIEN]

Des questions ou retours sont les bienvenus !
```

---

## Suivi dans validation_evidence.md

### Landing Page Metrics — [Date lancement]

**URL** :
**Date lancement** :
**Date cloture** :

| Metrique | Valeur | Seuil | Atteint ? |
|----------|--------|-------|-----------|
| Visites | | 1000 | [ ] |
| Inscriptions | | 100 | [ ] |
| Conversion | | 3% | [ ] |
| Borrowers | | 60 | [ ] |
| Providers | | 40 | [ ] |
| Budget >20€ | | 30 | [ ] |

**Verdict** : [ ] SEUIL ATTEINT (3 interviews equivalentes) / [ ] INSUFFISANT

**Preuves** :
- Screenshot analytics :
- Export CSV inscriptions :
- Analyse quali profils :

---

## Next Steps Post-Landing Page

Si seuil atteint (100+ inscrits, 3-5% conversion) :
1. Identifier 10 leads chauds (borrowers avec budget)
2. Contacter pour interviews approfondies (2 interviews = 2 points)
3. Lancer MVP Technique avec les plus motives (2 points)
4. Total : 3 + 2 + 2 = 7 points > 5 requis = Mom Test valide

Si seuil non atteint :
1. Analyser pourquoi (copy ? targeting ? canal ?)
2. Pivot sur Wizard of Oz ou Comportement Observe
3. Ou revenir a interviews classiques
