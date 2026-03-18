# Lilavati — Feature Roadmap

## The Complete Feature Universe + Open Source vs Paid Strategy

### The Framework

The open-core model follows one rule:

> **Open source the computation. Charge for the operations.**

- **Open source** = things developers need to evaluate and trust before paying
- **Paid API** = things developers need to run in production without ops burden
- **Premium** = things that require additional infrastructure (AI, webhooks, bulk, SLA)

---

## Feature Map

### Zone 1: Open Source Library (`pip install lilavati`)

These features drive adoption. They must be free because:
- They're derivable from public algorithms (no point gatekeeping math)
- Developers need to test accuracy before committing
- They generate GitHub stars, blog posts, Stack Overflow answers

| Feature | What It Does | Status | Difficulty |
|---------|-------------|--------|------------|
| Panchang (5 elements) | Tithi, Nakshatra, Yoga, Karana, Vara with transition times | Done | - |
| Sunrise/Sunset | Hindu rising method, location-aware, refraction-corrected | Done | - |
| Basic Muhurat Windows | Rahu Kalam, Yama Gandam, Gulika, Abhijit, Choghadiya | Done | - |
| Moon Data | Sidereal longitude, phase angle | Done | - |
| Festival Calculator | Astronomically computed Hindu festival dates (Diwali, Holi, etc.) | Phase 2 | Medium |
| Ekadashi/Vrat Dates | All 24 Ekadashis, Pradosh, Chaturthi, Amavasya, Purnima per year | Phase 2 | Medium |
| Sankranti Dates | Makar Sankranti, Ugadi, Vishu, Pongal, Bihu | Phase 2 | Medium |
| Hindu Month/Year | Lunar month name, Paksha, Vikram/Shaka Samvat year | Phase 2 | Easy |
| Planetary Positions | All 9 grahas in sidereal signs/nakshatras | Phase 2 | Easy |
| Rashi (Sign) | Which Rashi each planet occupies | Phase 2 | Easy |
| Regional Calendars | Tamil, Telugu, Bengali, Marathi, Gujarati, Kannada, Malayalam | Phase 2 | Hard |
| Shraddha Tithi | Death anniversary lunar date calculator | Phase 2 | Medium |
| Adhik Maas Detection | Leap lunar month detection | Phase 2 | Medium |

**Why these are free**: A developer needs to run `pip install lilavati` and verify "does this give me the right Diwali date?" before they'll trust your API in production. If the library is accurate, they become API customers.

### Zone 2: Free API Tier (1,000 requests/month)

Same computations as the library, but hosted. This tier exists so developers can:
- Prototype and test integration without self-hosting
- Build demos for their stakeholders
- Get addicted to the convenience before scaling

| Feature | Endpoint | Notes |
|---------|----------|-------|
| Daily Panchang | `GET /v1/panchang` | All 5 elements + sun data |
| Festival Dates | `GET /v1/festivals` | By year + region |
| Today Summary | `GET /v1/today` | Quick "what's happening now" |
| Muhurat Windows | `GET /v1/muhurat/windows` | Rahu Kalam, etc. |

### Zone 3: Paid API Tiers ($19–$299/month)

Features that add value beyond raw computation — things a developer could technically build themselves but won't because it's painful. **This is where the money is.**

| Feature | What It Does | Why It's Paid | Buyer |
|---------|-------------|---------------|-------|
| Kundali Generation | D1 (Rashi) + D9 (Navamsa) + all 16 Varga charts | Complex divisional chart math, chart rendering (SVG/JSON) | Astrology apps, matrimony |
| Kundali Matching | Ashtakoot Guna Milan (score/36), Dashakoot, Manglik Dosha | #1 revenue driver. Every matrimony app needs this. | Matrimony apps |
| Dasha System | Vimshottari (120-year), Yogini, Char Dasha periods | Multi-level Mahadasha→Antardasha→Pratyantardasha tree | Astrology consultation apps |
| Yoga Detection | 100+ classical yogas (Raj Yoga, Dhan Yoga, Gajakesari, etc.) | Requires encoding rules from BPHS/Saravali | Astrology apps |
| Dosha Analysis | Manglik/Kuja Dosha, Kaal Sarpa Dosha, Pitru Dosha | Specific chart pattern matching | Matrimony, astrology apps |
| Shadbala | 6-fold planetary strength calculation | Serious computational depth, professional use | Pro astrologer tools |
| Ashtakavarga | 8-source strength, Sarvashtakavarga, transit scoring | Professional transit prediction | Pro astrologer tools |
| Electional Muhurat | Best dates for wedding/griha pravesh/vehicle/business | Combines multiple Panchang factors + Kundali | PropTech, event planning, fintech |
| Transit Alerts (Webhooks) | Push events when planets change sign/nakshatra | Infrastructure feature — requires event loop, queue | Any app with push notifications |
| Varshaphal | Annual horoscope (solar return chart) | Tajika system, niche but high-value | Birthday feature in apps |
| Lal Kitab | Lal Kitab chart + remedies per house | Unique system, popular in North Bhārat | Astrology apps |
| PDF Reports | Branded Kundali/Matching PDF generation | White-label with customer's logo | Matrimony, astrology businesses |
| Batch Operations | Bulk matching (e.g., match 1 profile against 500) | Enterprise matrimony feature | Shaadi.com-scale apps |
| Multi-Language | Responses in Hindi, Tamil, Telugu, Bengali, etc. | Translation + localization of all terms | Consumer-facing apps |

### Zone 4: Premium Tier ($299+/month or per-query pricing)

These require additional infrastructure (LLM calls, real-time systems) and create the strongest lock-in.

| Feature | What It Does | Why It's Premium | Buyer |
|---------|-------------|-----------------|-------|
| LLM Interpretation | Natural language reading of any chart/transit/dasha | Requires Claude/GPT call per request + prompt engineering grounded in Sanskrit texts | AI astrology chatbots |
| Prashna Kundali | Horary chart — answer questions without birth data | Unique use case, great for chatbot UX | AI/chatbot builders |
| KP System | Krishnamurti Paddhati sub-lord calculations | Dedicated community, niche but loyal | KP-specific apps |
| Remedial Measures | Gemstone, mantra, donation recommendations per chart | Can tie to e-commerce (affiliate revenue potential) | Astrology + e-commerce |
| Vastu API | Floor plan direction → Vastu compliance score | Zero competition as an API, big PropTech demand | PropTech, real estate |
| Sanskrit Source Citations | Every computation linked to original BPHS/Saravali sloka | Authority signal, academic value | Education, serious astrology platforms |
| Real-Time Event Stream | SSE/WebSocket stream of astronomical events | Requires persistent connection infrastructure | Notification-heavy apps |
| Sade Sati Tracker | Saturn's 7.5-year transit cycle over Moon sign | Extremely popular fear-driven feature in Bhāratīya astrology | Every astrology app |

---

## Revenue Architecture

```
                ┌─────────────────────────────────┐
                │  PREMIUM ($299+/mo or per-query) │
                │  LLM Interpretation, Prashna,    │
                │  Vastu, KP, Remedies, SSE Stream │
                ├─────────────────────────────────┤
                │  PAID API ($19-$79/mo)           │
                │  Kundali, Matching, Dasha, Yoga, │
                │  Dosha, Muhurat, Transits,       │
                │  Webhooks, PDF, Multi-Language    │
                ├─────────────────────────────────┤
                │  FREE API (1,000 req/mo)         │
                │  Panchang, Festivals, Today,     │
                │  Basic Muhurat Windows            │
                ├─────────────────────────────────┤
                │  OPEN SOURCE LIBRARY (free)      │
                │  Panchang, Festivals, Sun/Moon,  │
                │  Muhurat, Regional Calendars,    │
                │  Planetary Positions              │
      ▲         └─────────────────────────────────┘
      │
Distribution         ──────────────────►       Revenue
(wider base                                    (narrower,
 = more users)                                  higher value)
```

---

## Revenue vs Adoption Matrix

| Feature | Revenue Potential | Adoption Potential | Build Order Priority |
|---------|-------------------|-------------------|---------------------|
| Festival Calendar | Low (hard to charge for dates) | Very High (every Bhāratīya app) | Phase 2 — build now for distribution |
| Kundali Matching | Very High (matrimony apps pay) | Medium | Phase 3 — build for revenue |
| Kundali Generation | High | Medium | Phase 3 — pairs with matching |
| Dasha System | High | Low (only astrology apps) | Phase 3 |
| LLM Interpretation | Very High (premium pricing) | Medium | Phase 4 — differentiator |
| Transit Alerts/Webhooks | High (infrastructure lock-in) | Low | Phase 3 |
| Vastu API | Medium (new market) | Low (niche) | Phase 4 — blue ocean |
| Regional Calendars | Low | High (HR/payroll/govt apps) | Phase 2 — build for distribution |
| Sade Sati Tracker | Medium | High (fear-driven, everyone checks) | Phase 3 — easy, high interest |

---

## Novel AI/LLM Features

Modern LLMs and AI advancements unlock capabilities that were impossible before. These are Lilavati's potential differentiators.

### 1. Computation-Grounded LLM Interpretation (The Killer Feature)

The #1 problem with AI + astrology today: **LLMs hallucinate planetary positions**. They're token predictors, not calculators. Every "AI astrology" chatbot that tries to compute charts gets positions wrong.

Lilavati's architecture is the exact solution:
- **Swiss Ephemeris computes** (accurate to arc-seconds)
- **LLM interprets** (natural language explanation of what the positions mean)

This is a defensible moat. Competitors building "AI astrology" without a proper computation engine will always hallucinate.

```
POST /v1/interpret
{
  "date": "2026-02-24",
  "location": "Delhi",
  "question": "Is today good for starting a business?"
}
```

Response combines exact Panchang data + LLM interpretation grounded in that data. The LLM never guesses positions — it receives them as structured context.

### 2. RAG on Sanskrit Source Texts

RAG (Retrieval-Augmented Generation) on religious/Sanskrit texts significantly reduces hallucinations. The corpus:

- **Brihat Parashara Hora Shastra (BPHS)** — the foundation of Vedic astrology
- **Surya Siddhanta** — astronomical computation methods
- **Muhurta Chintamani** — electional astrology rules
- **Phaladeepika** — predictive rules

When someone asks "why is this Tithi considered auspicious?", the LLM retrieves the actual verse from BPHS and cites it. No other competitor does this — they all just hardcode rules without source attribution. This builds trust with the knowledgeable Hindu audience.

### 3. Chart-to-Text Pipeline

Convert structured astrological data into LLM-readable narratives:

```
"On 2026-02-24, Moon is in Uttara Bhadrapada Nakshatra (ruled by Saturn),
Shukla Paksha Saptami. Sun-Moon angular distance is 84°. Rahu Kalam
falls during 15:00-16:30 IST. The Yoga is Siddha (auspicious)."
```

This structured text becomes the grounding context for any LLM — whether Claude, GPT, Gemini, or open-source models. Lilavati becomes the **computation layer** that every AI astrology product needs.

### 4. Agentic Multi-Step Reasoning

Modern LLM agent patterns enable complex queries that chain multiple computations:

- "When is the next Pushya Nakshatra on a Thursday with no Rahu Kalam overlap?"
  *(requires: Nakshatra search + Vara check + Rahu Kalam check, iterating forward)*
- "Find the best muhurat for a wedding in March 2026"
  *(requires: iterate days, check multiple criteria per classical rules, rank results)*
- "Compare my birth chart with this date's transit chart"
  *(requires: two chart computations + aspect analysis)*

Lilavati's computation engine becomes the **tool** that an LLM agent calls repeatedly. This is the LLM tool-use / function-calling pattern — having a reliable, fast computation API is essential for it to work.

### 5. The API-as-LLM-Tool Play

**The strategic insight**: as LLM tool-use becomes standard, every AI assistant that wants to answer "is today auspicious?" needs a reliable computation API to call. You don't need to build the AI product — you become the **tool that AI products call**.

```json
{
  "name": "get_panchang",
  "description": "Get accurate Bhāratīya calendar data for a date and location",
  "api": "https://api.lilavati.dev/v1/panchang"
}
```

This positions Lilavati like Wolfram Alpha is for math — the computation oracle that LLMs defer to instead of guessing.

### 6. Multimodal Possibilities (Future)

- **Kundli image generation**: Publication-quality North/South Bhāratīya chart diagrams from computation data
- **Voice AI backend**: Companies like Pandit.ai are building "voice AI astrologers" — they need a computation backend
- **Vastu from floor plans**: Multimodal LLMs can analyze floor plan photos + compass direction for Vastu analysis

### AI Feature Build Phases

| Phase | AI Feature | Effort |
|-------|-----------|--------|
| Phase 1 (done) | Structured JSON output (already done) | Done |
| Phase 2 | Chart-to-text generation | 1 week |
| Phase 3 | LLM interpretation endpoint (grounded) | 2 weeks |
| Phase 3 | MCP/OpenAI function spec for tool-use | 1 week |
| Phase 4 | RAG on Sanskrit texts | 3-4 weeks |
| Phase 4 | Agentic muhurat search | 2 weeks |

---

## Build Sequence

### Phase 2 (Next 3-4 weeks) — Distribution Features
- Festival calculator (astronomically computed, 30+ major festivals)
- Ekadashi/Vrat calendar
- Hindu month/year with Adhik Maas
- Regional calendar variants (at least Tamil, Telugu, Bengali, Hindi)
- Ship to PyPI v0.2.0

### Phase 3 (Weeks 7-12) — Revenue Features
- FastAPI hosted API with auth + billing
- Kundali generation (D1 + D9)
- Kundali matching (Ashtakoot — this is the money maker)
- Sade Sati detection (easy, high engagement)
- Transit alerts as webhooks

### Phase 4 (Month 4+) — Moat Features
- LLM interpretation layer
- Full Dasha system
- Yoga/Dosha detection
- Vastu API
- Sanskrit source citations

---

## The Key Insight

> The open-source library should contain everything needed to answer: **"What is happening in the Bhāratīya calendar?"** (Panchang, festivals, muhurat, regional calendars).
>
> The paid API should contain everything needed to answer: **"What does this mean for ME?"** (Kundali, matching, dasha, predictions, remedies, interpretations).
>
> Calendar data is infrastructure (free, drives adoption). Personal astrology is interpretation (paid, drives revenue).
>
> **That's the line.**
