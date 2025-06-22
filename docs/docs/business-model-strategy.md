# Business Model/Publisher Strategy

This document outlines Bluefelt's planned business structure, licensing framework, and publisher relations strategy based on early strategic planning.

## Foundation Structure

### Non-Profit Model Benefits

Bluefelt Foundation operates as a non-profit organization focused on engine development and open-source stewardship, while enabling publishers to maintain control and profit from their game IP.

**Key Advantages:**
- **Publisher Trust**: No competing commercial interests with game sales
- **Community Focus**: Mission-driven development prioritizes platform improvement over extraction
- **Long-term Stability**: Non-profit structure prevents hostile takeovers or mission drift
- **Tax Benefits**: Potential 501(c)(6) status for industry consortium benefits

### Legal Framework Options

| Structure | Pros | Cons | Best For |
|-----------|------|------|----------|
| **Standalone Non-Profit** | Full control, custom bylaws | Higher administrative overhead | Mature platform with dedicated staff |
| **Software Freedom Conservancy** | Established infrastructure, legal support | Less autonomy, shared resources | Early-stage development |
| **Linux Foundation Project** | Industry credibility, corporate sponsorship | Corporate governance focus | Enterprise adoption strategy |

## Licensing Strategy

### Two-Track IP Approach

**Engine & Tooling**: MIT/Apache-2.0 + Contributor License Agreement
- Maximizes developer adoption with permissive licensing
- CLA allows re-licensing if needed for future compatibility
- Foundation owns and maintains open-source engine

**Game Bundles**: Publisher's Digital Adaptation License (DAL)
- Publishers retain all IP ownership
- Limited digital distribution rights granted to Bluefelt
- Server authority prevents gameplay piracy/modification

### Digital Adaptation License (DAL) Framework

#### Grant of Rights
- **Scope**: Non-exclusive, worldwide, digital-only interactive adaptations
- **Term**: 3-year auto-renewal with 90-day opt-out notice
- **Derivative Works**: Rules-accurate adaptations; cosmetic skins with pre-approval

#### Revenue Models (Publisher Choice)

| Model | Mechanism | Publisher Share | Best For |
|-------|-----------|-----------------|----------|
| **Marketplace Sale** | Foundation storefront, license per lobby | 80-90% net | Medium-weight hobby titles |
| **Seat Licenses** | Bulk key sales for publisher distribution | 100% - cert fee | Kickstarter rewards, retail inserts |
| **Royalty Floor** | Free-to-play with minimum per play-hour payments | Variable floor | Gateway games seeking exposure |
| **Self-Hosted** | Publisher runs own Bluefelt server | 100% - cert fee | Large publishers with existing portals |

#### Security & Anti-Tamper
- **Hash Verification**: Engine rejects modified bundles (SHA-256 mismatch)
- **Server Authority**: All state changes server-side; clients cannot inject moves
- **Optional DRM**: Encrypted asset bundles for high-value art

## Publisher Pricing Strategy

### Fair Pricing Framework

**Host License Pricing**: 25-40% of physical MSRP, capped at $25, floor at $4.99

| Physical MSRP | Suggested Host Price | Rationale |
|---------------|---------------------|-----------|
| Under $20 (filler) | $4.99 - $6.99 | Coffee-money impulse purchase |
| $25-$50 (gateway) | $7.99 - $11.99 | ~$2-3 per player in typical 4p group |
| $55-$90 (expert) | $12.99 - $18.99 | Half the cost of physical copy |
| $100+ deluxe | $19.99 - $24.99 | Matches premium digital board games |

**Value Reinforcements:**
- Only host needs to purchase (free guest seats)
- Cross-platform entitlement (web, mobile, VR)
- Family sharing (multiple simultaneous lobbies)
- Seasonal sales (20-40% off during events)

### Revenue Split Example
```
Host Price: $14.99
- Payment Processing (Stripe): ~$0.75
- Platform Operations (10%): $1.42
- Publisher Royalty (85% gross): ~$12.82
```

## Publisher Relations Framework

### Quality Assurance Process
1. **Validator Pass**: Bundle compilation and spec compliance check
2. **Certification Loop**: Private test lobby for publisher sign-off  
3. **Production Whitelist**: Hash approval for live servers
4. **Update Process**: Art updates independent of gameplay changes

### Publisher Benefits
- **Direct Payment Flow**: Funds flow directly to publisher (minus transparent platform fee)
- **Real-time Analytics**: Dashboard with sales, play-hours, downloadable CSV
- **Brand Protection**: Server authority prevents rule modifications
- **Audit Trail**: Complete transaction history and replay logs

### Community & Moderation
- **Mod Policy**: Community can create appearance bundles but cannot alter server rules
- **Cultural Localization**: Engine supports external i18n files; publishers can sell language packs
- **Advisory Board**: Non-voting seats for publishers with multiple active titles

## Precedents and Validation

### Successful Non-Profit Revenue Models

| Organization | Model | Annual Scale | Key Success Factor |
|--------------|-------|--------------|-------------------|
| **ASCAP** (Music) | Blanket licenses → songwriter royalties | >$1B distributed | Transparent governance, 90% pass-through |
| **Copyright Clearance Center** | Business licenses → publisher payments | Major scale | Efficient bulk licensing |
| **Open Source Collective** | Sponsorship → project funding | Growing | Open finances, 10% host fee |
| **Ghost Foundation** | SaaS hosting → development funding | ~$7.5M ARR | Mission-driven, permissive code license |

### Key Success Patterns
1. **Transparent Platform Fee**: Fixed, published percentage builds trust
2. **Direct Payment Flow**: Funds never "held" by foundation 
3. **Member Governance**: Rights-holders have voice in platform direction
4. **Open Standards**: APIs and processes available for verification
5. **Mission Alignment**: Non-profit structure prevents conflicts of interest

## Implementation Roadmap

### Foundation Setup (Months 1-3)
- [ ] Choose legal structure and incorporate
- [ ] Draft bylaws and governance documents  
- [ ] File trademark and tax-exempt applications
- [ ] Set up payment processing (Stripe Connect)

### DAL Development (Months 2-4)
- [ ] Draft Digital Adaptation License template
- [ ] Legal review with game industry attorney
- [ ] Publisher feedback sessions and iteration
- [ ] Create certification process and tools

### Publisher Onboarding (Months 4-6)
- [ ] Build publisher dashboard MVP
- [ ] Pilot program with 2-3 mid-size publishers
- [ ] Refine processes based on feedback
- [ ] Launch public publisher program

## Success Metrics

- **Publisher Adoption**: Number of signed DALs and active game catalogs
- **Revenue Efficiency**: Publisher satisfaction with royalty flow and reporting
- **Platform Health**: Community growth and developer ecosystem expansion
- **Mission Alignment**: Open-source contribution rate and foundation transparency

This business model balances the need for sustainable platform development with publisher profitability and community growth, creating a foundation for long-term ecosystem success.