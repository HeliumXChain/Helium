# v0 Prompt — Helium Communities Landing Page

**URL v0** : https://v0.dev

---

## Prompt Principal

```
Create a modern, dark-themed landing page for "Helium Communities" — a P2P compute sharing network for developers.

HERO SECTION:
- Headline: "Borrow RAM/GPU from developers you trust"
- Subheadline: "Fine-tune your LLMs without paying AWS. Share resources, earn credits."
- Email capture form with submit button "Join the beta"
- Subtext: "100+ devs already on the list. Limited beta access."
- Background: Dark gradient (slate/black) with subtle grid pattern

PROBLEM SECTION ("The Pain"):
- Title: "You know this pain?"
- 4 cards with pain points:
  1. "Colab disconnects you" — "Your training stops after 2h. You lose hours."
  2. "AWS costs $300/month" — "Fine-tuning = $50-100 on AWS. Too expensive for indies."
  3. "Your laptop crashes" — "Can't fine-tune models > 7B on 16GB RAM."
  4. "You don't dare to ask" — "You know a dev with a powerful PC. But asking is complicated."
- Card style: Dark cards with red/pink accent for pain

SOLUTION SECTION ("Helium Communities"):
- Title: "Helium Communities — Trusted Sharing"
- 4 feature cards with icons:
  1. "Trusted Network" — "Web of Trust. Borrow only from verified devs in your community."
  2. "Encrypted Tunnel" — "WireGuard (military-grade encryption). Zero MITM risk."
  3. "Isolated VM" — "Firecracker microVM. Borrower sees only the VM, never your files."
  4. "Local Credits" — "Lend today, borrow tomorrow. No blockchain, no speculation."
- Card style: Dark cards with purple/blue gradient accents

HOW IT WORKS:
- Title: "How it works"
- Two paths side by side or alternating:

BORROWER (4 steps):
1. "Find a provider" — "Search in your community for a dev with available resources."
2. "Connect" — "WireGuard encrypted tunnel between your machines. Setup in 30 seconds."
3. "Execute" — "Fine-tune your model in the isolated VM on the provider's machine."
4. "Pay" — "Pay the provider in community credits or cash."

PROVIDER (4 steps):
1. "List resources" — "Declare your available resources."
2. "Accept requests" — "Accept requests from trusted connections."
3. "Earn credits" — "Earn credits for future use."
4. "Stay in control" — "Full control with VM kill switch."

TESTIMONIAL:
- Quote: "I fine-tuned Llama-3 8B in 2h on a friend's workstation. Zero AWS config."
- Author: "Indie dev, Paris"

FAQ SECTION (expandable):
- Q: "What's the difference with RunPod/AWS?"
  A: "RunPod = strangers + expensive. Helium = trusted friends + 50% cheaper."
- Q: "Is it secure?"
  A: "WireGuard tunnel (Curve25519 encryption) + isolated Firecracker VM."
- Q: "Is it free?"
  A: "Free beta. Then direct payments between members (credits or cash)."
- Q: "When available?"
  A: "Open beta Q2 2026. Sign up for priority access."

FINAL CTA:
- Title: "Join the beta"
- Text: "100+ devs already on the list. Limited access."
- Form: Email + Profile select ("I need GPU/RAM" / "I have resources" / "Both")
- Button: "Join the beta"

FOOTER:
- "Helium Communities — 2026"

STYLE:
- Dark theme (slate/black background)
- Purple/blue gradient accents (#667eea to #764ba2)
- Modern sans-serif font (Inter or similar)
- Responsive design
- Smooth animations on scroll
- Glassmorphism cards with subtle borders

TECH:
- React + Tailwind CSS
- Client-side form handling
- Ready for Next.js deployment
```

---

## Prompts de Refinement (si besoin)

```
Make the hero section more bold with larger typography and a gradient text effect on the headline.
```

```
Add a subtle animated background with floating particles or gradient orbs in the hero.
```

```
Make the cards in the problem section have a hover effect that lifts them up.
```

```
Add a visual diagram or illustration showing the P2P connection between borrower and provider.
```

```
Change the primary color to a more vibrant purple and add glowing effects on the CTA buttons.
```

---

## Post-Creation Checklist

- [ ] Export code from v0
- [ ] Create Next.js project or copy to existing project
- [ ] Replace placeholder form action with Formspree URL
- [ ] Test form submission
- [ ] Deploy to Vercel
- [ ] Add Plausible or Google Analytics
- [ ] Test mobile responsiveness
- [ ] Share URL with team

---

## Form Integration (Formspree)

After v0 export, replace the form with:

```tsx
<form action="https://formspree.io/f/YOUR_FORM_ID" method="POST">
  <input type="email" name="email" placeholder="Your email" required />
  <select name="profile" required>
    <option value="">I am...</option>
    <option value="borrower">Borrower — I need GPU/RAM</option>
    <option value="provider">Provider — I have resources</option>
    <option value="both">Both</option>
  </select>
  <button type="submit">Join the beta</button>
</form>
```

---

## Deployment Steps

1. Export from v0 → "Copy code" or "Deploy to Vercel"
2. If "Copy code":
   - Create new Next.js project: `npx create-next-app helium-landing`
   - Replace `app/page.tsx` with v0 code
   - Run `npm run dev` to test locally
   - Push to GitHub
   - Import to Vercel
3. If "Deploy to Vercel":
   - Connect GitHub account
   - Vercel auto-deploys
   - Get live URL instantly

---

## Analytics Setup

Add to `<head>` in `app/layout.tsx`:

```tsx
<script defer data-domain="your-domain.vercel.app" src="https://plausible.io/js/script.js"></script>
```

Or Google Analytics 4 if preferred.
