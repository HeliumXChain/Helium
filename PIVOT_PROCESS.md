# Pivot Process — Validation en 1 Jour avec Desk Research

**Objectif** : Pivoter rapidement si le Mom Test echoue, en validant une nouvelle hypothese en 1 jour maximum.

**Philosophie** : Claude et les AI modernes permettent de recreer entierement un projet et de pivoter en 1 jour. Ce process formalise cette vitesse.

---

## Principe Fondamental

```
IF Desk Research = NO-GO:
  ACTION: Pivot immédiat (meme jour)
  ACTION: Nouveau Desk Research (4h max)
  ACTION: Nouveau prototype (4h max)
  DO NOT: Persister sur une idee invalidee
```

---

## Timeline Pivot (1 Jour = 8 Heures)

### Matin (4h) — Validation Nouvelle Hypothese

**Heure 0:00-0:30 : Definition du Pivot**
- [ ] Identifier pourquoi l'ancienne idee a echoue
- [ ] Definir la nouvelle hypothese probleme
- [ ] Choisir le segment cible pivot
- [ ] Documenter dans `pivot_log.md`

**Heure 0:30-2:30 : Desk Research (Nouvelle Hypothese)**
- [ ] 5+ sources concurrents
- [ ] 5+ preuves communautaires
- [ ] 3+ sources marche
- [ ] Template : `desk_research_report.md`

**Heure 2:30-3:00 : Analyse et Verdict**
- [ ] Remplir checklist validation
- [ ] Verdict : GO / NO-GO / PIVOT ENCORE
- [ ] Si NO-GO : Nouveau pivot (iteration)
- [ ] Si GO : Passer a l'apres-midi

**Heure 3:00-4:00 : Mise a jour Documentation**
- [ ] Mettre a jour `decision-memo.md`
- [ ] Creer nouveaux scripts Mom Test (si applicable)
- [ ] Mettre a jour `validation_evidence.md`
- [ ] Commit des changements

### Apres-midi (4h) — Prototype MVP

**Heure 4:00-5:00 : Conception MVP**
- [ ] Definir le scope MVP (1 feature)
- [ ] Choisir la stack technologique
- [ ] Creer diagramme architecture (ASCII)
- [ ] Lister les fichiers necessaires

**Heure 5:00-7:00 : Development MVP**
- [ ] Setup projet (init repo, deps)
- [ ] Implementer core feature
- [ ] Tests basiques
- [ ] README avec instructions

**Heure 7:00-8:00 : Validation Technique + Documentation**
- [ ] Tester le MVP (cas d'usage)
- [ ] Documenter bugs/limitations
- [ ] Mettre a jour `SESSION_SUMMARY.md`
- [ ] Commit final

---

## Checkpoints Decisionnels

### Checkpoint 1 : Desk Research (Midi)

```
IF desk_research.verdict == "GO":
  CONTINUE -> Prototype MVP

ELIF desk_research.verdict == "NO-GO":
  PIVOT_AGAIN -> Nouvelle hypothese (Heure 0)

ELIF desk_research.verdict == "PIVOT":
  ADJUST -> Modifier l'hypothese (Heure 0-1)
```

### Checkpoint 2 : Prototype (Soir)

```
IF mvp.fonctionnel == True:
  SUCCESS -> Lancer MVP Technique / Landing Page

ELIF mvp.bloquant == True:
  SIMPLIFY -> Reduire scope (Heure 5-6)

ELSE:
  PIVOT_TECH -> Changer stack/approche (Heure 4)
```

---

## Template Pivot Log

```markdown
# Pivot Log — [Date]

## Pivot #[N]

**Ancienne Hypothese** : [Description]
**Raison Echec** : [Desk Research ou Mom Test result]

**Nouvelle Hypothese** : [Description]
**Nouveau Segment** : [Cible]
**Nouveau Probleme** : [Pain point]

### Desk Research (Nouvelle Hypothese)

**Verdict** : GO / NO-GO / PIVOT
**Sources** : [Nombre]
**Preuves comportementales** : [Nombre]
**Confiance** : High / Medium / Low

### Prototype MVP

**Scope** : [1 feature]
**Stack** : [Technologies]
**Status** : Fonctionnel / Bloquant / Abandonne

### Decision

- [ ] GO — Valider avec MVP Technique
- [ ] NO-GO — Pivoter encore
- [ ] ADJUST — Modifier et retester

**Prochaines etapes** : [Actions]
```

---

## Outils pour Vitesse Maximale

### AI Tools

| Outil | Usage | Gain de temps |
|-------|-------|---------------|
| **Claude/Cursor** | Code generation | 10x vs manuel |
| **v0** | Landing page UI | 1h vs 1 journee |
| **Perplexity** | Desk Research | 2h vs 2 jours |
| **Whimsical** | Diagrammes | 15min vs 1h |
| **GitHub Copilot** | Code completion | 2x sur boilerplate |

### Templates Prets

- `desk_research_report.md` — Analyse marche
- `validation_evidence.md` — Preuves comportementales
- `landing_page_plan.md` — Plan landing page
- `x_thread.md` — Thread Twitter
- `decision-memo.md` — Decision produit

---

## Metriques de Succes Pivot

### Desk Research
- [ ] 10+ sources verifiees en < 2h
- [ ] 3+ preuves comportementales
- [ ] Verdict GO avec confiance High/Medium

### MVP
- [ ] 1 feature fonctionnelle en < 3h
- [ ] Code propre et documente
- [ ] Tests basiques passent

### Documentation
- [ ] `pivot_log.md` a jour
- [ ] `SESSION_SUMMARY.md` mis a jour
- [ ] Commit git avec message descriptif

---

## Exemple de Pivot Reussi

### Scenario : Helium Communities -> Helium Core

**Matinee (4h)** :
1. **0:00-0:30** : Desk Research montre que P2P trust network est deja couvert par Bittensor
2. **0:30-2:30** : Nouveau Desk Research sur blockchain PoUW pour AI training
3. **2:30-3:00** : Verdict GO — Marche $77B, gap identifie (verification zk-ML)
4. **3:00-4:00** : Mise a jour docs (decision-memo, spec)

**Apres-midi (4h)** :
1. **4:00-5:00** : Conception — Core PoUW en Rust, verification training
2. **5:00-7:00** : Dev — Setup Rust, structure modulaire, 1 proof of concept
3. **7:00-8:00** : Test + Doc — PoC fonctionnel, README, commit

**Resultat** : Nouveau projet valide et prototype fonctionnel en 1 jour.

---

## Anti-Patterns a Eviter

### ❌ Ne PAS faire

1. **Attendre 1 semaine** pour decider du pivot
2. **Persister** sur une idee apres 3 pivots consecutifs NO-GO
3. **Sauter le Desk Research** et coder direct (gaspillage)
4. **Over-engineer** le MVP (plusieurs features)
5. **Negliger la doc** (on oublie pourquoi on a pivote)

### ✅ FAIRE

1. **Echouer vite** — 4h de research max, pas 4 jours
2. **Documenter** — Chaque pivot est un apprentissage
3. **Simplifier** — 1 feature MVP, pas 10
4. **Automatiser** — Scripts, templates, checklists
5. **Iterer** — Pivot rapide, test, pivot encore si besoin

---

## Integration avec les Regles AGENTS.md

### Rule 2 : Mom Test Gate

Desk Research est une alternative valide au Mom Test quand :
- Le Mom Test est impossible (pas d'acces aux users)
- Le Mom Test a echoue (0 interviews valides)
- Besoin de pivot rapide (vitesse critique)

### Rule 4 : Progress Tracking

Chaque pivot doit etre documente dans `SESSION_SUMMARY.md` :
- Pourquoi le pivot ?
- Quelle nouvelle hypothese ?
- Desk Research verdict ?
- MVP status ?

### Rule 17 : Deep Understanding

Meme en pivot rapide, l'agent DOIT :
- Expliquer pourquoi l'ancienne idee a echoue
- Justifier la nouvelle hypothese avec evidences
- Montrer le lien logique entre les deux

---

## Prochaines Etapes

1. **Lire ce process** avec ton AI agent
2. **Pratiquer** sur un faux pivot (exercice)
3. **Appliquer** des que le besoin emerge
4. **Ameliorer** le process avec l'experience

---

**Remember** : Claude + bon process = pivot en 1 jour. Pas en 1 mois.
