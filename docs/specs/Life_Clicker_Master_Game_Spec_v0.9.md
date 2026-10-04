# Life Clicker — Master Game Specification v0.9

Baseline: Game 7.0.7 / Save Version 12 / Go Launcher 2.1.1 stable / Rust Launcher 3.0.0 candidate.

Purpose: establish one authoritative design baseline and expose unresolved design holes.

# 1. Executive Summary

Life Clicker is a deep, menu/click-driven life and economic simulation designed for long-term progression. The player builds a life across skills, employment, education, finances, business ownership, assets, risk, and legacy. The design intentionally favors understandable systems, explicit requirements, visible financial flow, and save safety over hidden complexity.

## 1.1 Core Design Pillars

| Pillar | Specification |

| --- | --- |

| Depth without obscurity | Systems may be complex, but requirements, blockers, costs, and consequences must be visible. |

| Long-term progression | A character should have meaningful goals from first job through wealth, business scale, and legacy. |

| Realistic financial flow | Income, expenses, capital investment, debt, assets, and net worth should behave coherently. |

| Player agency | The player chooses careers, businesses, skill paths, risk appetite, and financial strategy. |

| Consequences, not arbitrary punishment | Failure should be understandable, recoverable where reasonable, and linked to player choices. |

| Save permanence | Old characters and saves remain usable as the game evolves. |

| Low-friction updates | Normal releases should be detected and installed inside Life Clicker after the final launcher migration. |

| Boredom and confusion are bugs | If a system is technically functional but tedious or unclear, it is not finished. |



# 2. Specification Authority and Status

| Status | Meaning |

| --- | --- |

| CANONICAL | Explicitly approved rule and/or established release behavior that should not change casually. |

| TARGET | Agreed direction or intended end-state, but not necessarily fully implemented. |

| PROPOSED | Recommended default supplied to fill a hole; requires approval. |

| OPEN | No final decision exists yet. |

| VERIFY | Believed to exist but should be checked against source before relying on it. |



## 2.1 Conflict Resolution Order

1. Most recent explicit design decision by the owner.

2. This master specification after that decision has been incorporated.

3. Latest authoritative game source/release package.

4. Older notes, prior drafts, or provisional summaries.

# 3. Player Experience and Core Loop

- Choose a goal: employment, education, skill growth, financial milestone, business expansion, asset purchase, or another life objective.

- Take direct actions: train skills, apply for jobs, accept offers, transfer money, invest in a business, buy assets, choose upgrades, accept contracts, or manage risk.

- Advance time only when the selected action logically requires time.

- Receive outcomes through cash flow, XP, qualifications, reputation, assets, offers, contracts, events, and consequences.

- Reassess blockers and opportunities from a persistent Life Dashboard.

- Build toward higher autonomy: better jobs, scalable businesses, stronger finances, and eventually legacy progression.



## 3.1 Progression Layers

| Layer | Horizon | Examples |

| --- | --- | --- |

| Immediate | Clicks/actions | Skill XP, applications, transfers, purchases, management choices. |

| Short-term | Minutes to days | Job shifts, education/training actions, contracts, business operations, recovery. |

| Mid-term | Weeks to months | Promotions, credentials, business growth, major assets, credit/reputation. |

| Long-term | Years / character lifetime | Career mastery, wealth, multiple businesses, retirement, legacy, new character. |

| Meta | Across characters | Click Mastery and any approved inherited legacy systems. |



# 4. Time, Calendar, Energy, and Scheduling

## 4.1 Required Time-System Behaviors

# 5. Character, Life State, and Needs

## 5.1 Recommended Character State Model

| State domain | Contents | Status |

| --- | --- | --- |

| Identity | Name, age, background, optional appearance/profile | OPEN / PROPOSED |

| Core condition | Energy/sleep; optional health/stress/morale depending scope | PARTIAL |

| Progression | 35 trainable skills + Click Mastery derived from completed deaths | CANONICAL |

| Credentials | Education, licenses, certifications, training | OPEN structure |

| Economy | Cash, bank, illicit cash, debts, assets, net worth | PARTIAL |

| Employment | Current career, employer, pay, performance, tenure, offers | PARTIAL |

| Businesses | Ownership, company cash, employees, upgrades, assets, contracts | PARTIAL |

| Legal/risk | Heat, criminal consequences, records if approved | PARTIAL / OPEN |

| Legacy | Deaths completed, Click Mastery multiplier, future inheritance rules | PARTIAL |



# 6. Skills and Direct Training

## 6.1 Current General Skill Catalog

| # | Skill |

| --- | --- |

| 1 | Communication |

| 2 | Sales |

| 3 | Leadership |

| 4 | Management |

| 5 | Mechanical |

| 6 | Logistics |

| 7 | Driving |

| 8 | Construction |

| 9 | Technology |

| 10 | Healthcare |

| 11 | Finance |

| 12 | Agriculture |

| 13 | Investigation |

| 14 | Emergency Response |

| 15 | Discipline |

| 16 | Tactics |

| 17 | Unarmed Combat |

| 18 | Defense |

| 19 | Athletics |

| 20 | Awareness |

| 21 | Stealth |

| 22 | Deception |

| 23 | Street Smarts |

| 24 | Criminal Planning |

| 25 | Getaway Driving |

| 26 | Forgery |

| 27 | Coaching |

| 28 | Customer Service |

| 29 | Administration |

| 30 | Security |

| 31 | Legal |

| 32 | Culinary |

| 33 | Fitness |

| 34 | Vehicle Maintenance |

| 35 | Money Laundering |



## 6.2 Skill System Rules

# 7. Education, Training, Licenses, and Credentials

## 7.1 Credential Design Requirements

# 8. Career System

## 8.1 Career Offer Lifecycle

## 8.2 Career Content Requirements

# 9. Business Ownership and Operations

## 9.1 Supported / Planned Business Industries

| # | Industry |

| --- | --- |

| 1 | Lawn Care |

| 2 | Crop Farm |

| 3 | Machine Shop |

| 4 | Repair Garage |

| 5 | Foundry |

| 6 | Combat Gym |

| 7 | Criminal Syndicate (abstract fictional systems) |

| 8 | Online Retail |

| 9 | Restaurant / Cafe |

| 10 | Cleaning |

| 11 | Courier |

| 12 | Trucking |

| 13 | Construction |

| 14 | IT Services |

| 15 | Software Studio |

| 16 | Security |

| 17 | Accounting |

| 18 | Fitness Gym |

| 19 | Warehouse |



## 9.2 Canonical Financial Separation

| Flow | Cash movement | Accounting treatment |

| --- | --- | --- |

| Owner contribution | Personal cash → company cash | Equity/contribution; not revenue |

| Revenue | Customer/contract sales → company cash | Business revenue |

| Operating expense | Payroll, rent, utilities, insurance, maintenance, software, marketing, etc. | OPEX; reduces operating profit |

| Capital expenditure | Equipment, vehicle, facility, major permanent upgrade | CAPEX; recorded separately from monthly OPEX |

| Owner withdrawal/distribution | Company cash → owner, if/when supported | Must be explicit and tax/accounting-aware once tax model exists |

| Debt financing | Loan proceeds → company cash | OPEN; must not be misclassified as revenue |



## 9.3 Expense Categories

## 9.4 Upgrade Architecture

## 9.5 Contract System

# 10. Personal Economy and Wealth

## 10.1 Required Money Types

| Type | Purpose | Status |

| --- | --- | --- |

| Personal cash | Liquid on-hand legal money. | CANONICAL |

| Bank | Legal deposited money. | CANONICAL |

| Illicit cash | Separate proceeds from abstract underworld activity. | CANONICAL |

| Business cash | Tracked per business, separate from personal funds. | CANONICAL |

| Debt | Loans/credit/mortgages. | OPEN / TARGET |

| Investments | If approved, market or retirement assets. | OPEN |



# 11. Assets, Market, Inventory, Housing, and Vehicles

## 11.1 Asset Principles

# 12. Abstract Crime, Underworld Progression, and Legal Risk

## 12.1 Crime Action Card Requirements

# 13. Relationships, Household, and Family

# 14. Health, Insurance, Stress, and Recovery

# 15. Events, Reputation, and World Variability

# 16. Character Death, Legacy, and Click Mastery

# 17. UI/UX, Navigation, and Information Design

## 17.1 Recommended Primary Navigation

| Screen | Contents |

| --- | --- |

| Life | Dashboard, status, goals, schedule, activity |

| Skills | Training and progression |

| Career | Current job, applications, offers, career tree |

| Education | School, credentials, licenses |

| Business | Owned companies, staff, finances, upgrades, contracts |

| Finance | Cash/bank, debts, investments, taxes, net worth |

| Market | Assets, vehicles, equipment, property as approved |

| Underworld | Abstract crime/mastery/Heat/legal risk |

| Legacy | Character history, deaths, Click Mastery, inheritance as approved |

| Settings | Accessibility, update, save/recovery, developer information where appropriate |



## 17.2 Action-Card Standard

# 18. Content and Data Architecture

# 19. Saves, Migrations, Backups, and Recovery

## 19.1 Save-Safety Invariants

# 20. Launcher, Update, and Development Architecture

## 20.1 Intended Release Flow

# 21. QA, Regression, and Release Gates

| Gate | Minimum requirement |

| --- | --- |

| Source integrity | Build from latest authoritative source; prevent release/source drift. |

| Syntax/build | Rust/Go as applicable, JS syntax, UTF-8, required files. |

| Save migration | Upgrade old fixtures to current schema; verify preserved state. |

| Gameplay regression | Changed and adjacent systems still behave correctly. |

| Economy integrity | No double counting, negative impossible flows, or CAPEX/OPEX confusion. |

| Update integrity | Size/hash/version validation, staging, rollback, recovery. |

| UI clarity | Blockers/reasons visible; no dead buttons or endless “starting” states. |

| Windows acceptance | Run actual release candidate in native Windows environment. |

| Dev isolation | Port 8766/test saves cannot mutate production state on 8765. |



# 22. Balance and Pacing Framework

## 22.1 Balance Principles

# 23. Accessibility, Settings, and Player Control

# 24. Distribution and Product Strategy

# 25. v1.0 Content Completion Definition

# 26. Highest-Priority Holes to Resolve First

| ID | Area | Decision |

| --- | --- | --- |

| LC-GAP-004 | Time | Decide exact relationship between real-world time, in-game calendar time, and action duration. |

| LC-GAP-005 | Time | Define pause/offline behavior for jobs, contracts, businesses, travel, recovery, and events. |

| LC-GAP-006 | Time | Define sleep/energy regeneration rules and whether sleeping advances game calendar. |

| LC-GAP-008 | Character | Define character creation fields: name, age, location, background, starting education, traits, appearance. |

| LC-GAP-011 | Character | Define base stats/needs beyond energy and sleep: health, stress, hunger, morale, hygiene, etc. |

| LC-GAP-012 | Character | Define aging, lifespan, natural death, and retirement rules. |

| LC-GAP-014 | Skills | Define the XP curve/formula for uncapped skills. |

| LC-GAP-015 | Skills | Define training-click XP per click and how it scales with skill level. |

| LC-GAP-019 | Education | Define education levels and credentials (GED/high school, certificates, associate, bachelor, graduate, trade school). |

| LC-GAP-021 | Education | Define whether education consumes game time while skill training does not. |

| LC-GAP-023 | Career | Finalize complete career-tree catalog and branches. |

| LC-GAP-024 | Career | Define exact promotion probability formula and minimum non-zero chance. |

| LC-GAP-025 | Career | Define job performance calculation and contributing factors. |

| LC-GAP-032 | Business | Finalize startup requirements and starting capital for every industry. |

| LC-GAP-033 | Business | Define the canonical industry-specific upgrade catalog and permanent upgrade IDs. |

| LC-GAP-034 | Business | Define employee role taxonomy, wages, productivity, scheduling, hiring/firing, and turnover. |

| LC-GAP-037 | Business | Define inventory/raw-material procurement model for each relevant industry. |

| LC-GAP-043 | Business | Define business sale valuation formula including assets, profit, contracts, reputation, debt, and capital capability. |

| LC-GAP-044 | Business | Define contract capacity reservation, deadlines, penalties, quality thresholds, and repeat-client logic. |

| LC-GAP-046 | Economy | Define personal income tax, payroll tax, sales tax, property tax, and whether taxes are simplified or jurisdictional. |

| LC-GAP-048 | Economy | Define banking products: checking, savings, interest, fees, credit cards, loans, mortgages. |

| LC-GAP-050 | Economy | Define investment system scope: stocks, funds, bonds, retirement accounts, real estate, or none. |

| LC-GAP-053 | Assets | Define housing tiers, rent/buy, mortgages, utilities, maintenance, and property appreciation. |

| LC-GAP-054 | Assets | Define vehicle catalog, purchase/finance, insurance, fuel, maintenance, reliability, and depreciation. |

| LC-GAP-058 | Crime | Define Heat decay and escalation formula. |

| LC-GAP-059 | Crime | Define detection/arrest probability formula and hard minimum risk. |

| LC-GAP-060 | Crime | Define legal consequences in abstract terms: fines, temporary lockout/incarceration, record, career/business effects. |

| LC-GAP-063 | Relationships | Decide whether friendships, romance, marriage, children, and family are in scope. |

| LC-GAP-067 | Health | Define healthcare system depth: insurance, doctors, prescriptions as abstract costs, emergencies, preventive care. |

| LC-GAP-073 | Legacy | Define what causes a completed “character death” for Click Mastery and whether natural death and player-selected retirement both count. |

| LC-GAP-074 | Legacy | Define inheritance rules between characters and what persists besides Click Mastery. |

| LC-GAP-076 | UI | Finalize permanent HUD fields and responsive layouts for desktop/mobile. |

| LC-GAP-077 | UI | Define menu map and exact screen hierarchy. |

| LC-GAP-080 | UI | Define tutorial/onboarding path and when advanced systems unlock. |

| LC-GAP-081 | Technical | Finalize Rust Launcher 3.0 production acceptance and migration from Go 2.1.1. |

| LC-GAP-082 | Technical | Define canonical source-of-truth flow: game/index.html -> package -> stable manifest. |

| LC-GAP-083 | Technical | Define permanent ID/versioning rules for every content type (skills, careers, upgrades, items, contracts, events). |

| LC-GAP-085 | Technical | Define automated compatibility test matrix across old save versions. |

| LC-GAP-087 | Balance | Define target time-to-first job, first $10k, first business, first $100k net worth, first $1M net worth, and late-game milestones. |

| LC-GAP-092 | Content | Define content quantity targets per career branch, business industry, event category, contract class, and asset class for v1.0. |

| LC-GAP-093 | Content | Define location/world model: one abstract region, selectable cities, or a geographic map. |



# 27. Master Gap Register

| ID | Area | Decision needed | Priority | Why it matters |

| --- | --- | --- | --- | --- |

| LC-GAP-001 | Core | Define the exact win condition, if any: open-ended life simulation only, milestone victory, legacy score, or multiple endings. | P1 | Sets long-term motivation and UI messaging. |

| LC-GAP-002 | Core | Define the intended average session length and target daily/weekly play cadence. | P1 | Needed to tune timers, clicks, offers, and progression pacing. |

| LC-GAP-003 | Core | Define what “one year of meaningful progression” means in measurable content terms. | P1 | Prevents late-game content drought. |

| LC-GAP-004 | Time | Decide exact relationship between real-world time, in-game calendar time, and action duration. | P0 | Foundational for every scheduled action. |

| LC-GAP-005 | Time | Define pause/offline behavior for jobs, contracts, businesses, travel, recovery, and events. | P0 | Prevents inconsistent progression and exploits. |

| LC-GAP-006 | Time | Define sleep/energy regeneration rules and whether sleeping advances game calendar. | P0 | Blocks needs loop and scheduling. |

| LC-GAP-007 | Time | Define missed-shift, lateness, overtime, and schedule-conflict rules. | P1 | Required for realistic careers. |

| LC-GAP-008 | Character | Define character creation fields: name, age, location, background, starting education, traits, appearance. | P0 | Determines starting state and replay variety. |

| LC-GAP-009 | Character | Define starting age and minimum/maximum starting age. | P1 | Impacts career and education timelines. |

| LC-GAP-010 | Character | Decide whether permanent traits/aptitudes exist separately from trainable skills. | P1 | Controls build identity and replayability. |

| LC-GAP-011 | Character | Define base stats/needs beyond energy and sleep: health, stress, hunger, morale, hygiene, etc. | P0 | Major life-sim scope decision. |

| LC-GAP-012 | Character | Define aging, lifespan, natural death, and retirement rules. | P0 | Required for the legacy/death system. |

| LC-GAP-013 | Character | Define injury, illness, disability, and recovery scope. | P1 | Connects healthcare, work, insurance, and risk. |

| LC-GAP-014 | Skills | Define the XP curve/formula for uncapped skills. | P0 | Needed for consistent balance. |

| LC-GAP-015 | Skills | Define training-click XP per click and how it scales with skill level. | P0 | Directly controls click progression. |

| LC-GAP-016 | Skills | Define skill decay, if any. | P2 | Changes long-term maintenance burden. |

| LC-GAP-017 | Skills | Define cross-skill synergy rules and whether skill checks use one skill or weighted combinations. | P1 | Needed for jobs, contracts, business, and events. |

| LC-GAP-018 | Skills | Define exact 11 crime mastery names and their relationship to the 35 general skills. | P1 | Current system count exists but taxonomy needs canonicalization. |

| LC-GAP-019 | Education | Define education levels and credentials (GED/high school, certificates, associate, bachelor, graduate, trade school). | P0 | Career prerequisites depend on it. |

| LC-GAP-020 | Education | Define tuition, duration, admissions requirements, failure/withdrawal, and financing. | P1 | Required for economy/career realism. |

| LC-GAP-021 | Education | Define whether education consumes game time while skill training does not. | P0 | Core time-model consistency. |

| LC-GAP-022 | Education | Define licensing/certification renewal and expiration. | P2 | Impacts professional career upkeep. |

| LC-GAP-023 | Career | Finalize complete career-tree catalog and branches. | P0 | One of the largest content gaps. |

| LC-GAP-024 | Career | Define exact promotion probability formula and minimum non-zero chance. | P0 | Current rule exists conceptually, formula does not. |

| LC-GAP-025 | Career | Define job performance calculation and contributing factors. | P0 | Needed for promotions, raises, firing, and offers. |

| LC-GAP-026 | Career | Define raise cadence and raise-size formula separately from title promotions. | P1 | Needed for salary progression. |

| LC-GAP-027 | Career | Define employer generation: industries, sizes, pay bands, benefits, culture, stability. | P1 | Supports switching employers and role availability. |

| LC-GAP-028 | Career | Define firing, layoffs, quitting, notice periods, references, and rehire rules. | P1 | Completes employment lifecycle. |

| LC-GAP-029 | Career | Define work schedule types: fixed, rotating, nights, weekends, flexible, remote. | P1 | Connects time system and quality of life. |

| LC-GAP-030 | Career | Define benefits: health insurance, retirement match, PTO, bonuses, commissions, overtime. | P1 | Material to realistic job comparison. |

| LC-GAP-031 | Career | Define unemployment benefits and job-search friction, if any. | P2 | Economic safety-net decision. |

| LC-GAP-032 | Business | Finalize startup requirements and starting capital for every industry. | P0 | Required for industry balance. |

| LC-GAP-033 | Business | Define the canonical industry-specific upgrade catalog and permanent upgrade IDs. | P0 | Current generic tracks are transitional. |

| LC-GAP-034 | Business | Define employee role taxonomy, wages, productivity, scheduling, hiring/firing, and turnover. | P0 | Major realism system. |

| LC-GAP-035 | Business | Define manager automation scope and manager skill/quality effects. | P1 | Required for scaling beyond micromanagement. |

| LC-GAP-036 | Business | Define physical facility tiers, capacity, rent/buy choices, and relocation. | P1 | Needed for growth and capital investment. |

| LC-GAP-037 | Business | Define inventory/raw-material procurement model for each relevant industry. | P0 | Required for COGS and production realism. |

| LC-GAP-038 | Business | Define pricing strategy and demand elasticity controls. | P1 | Core profitability lever. |

| LC-GAP-039 | Business | Define competition model and market-share calculations. | P1 | Prevents business growth from being purely deterministic. |

| LC-GAP-040 | Business | Define debt/financing/loans/interest for businesses. | P1 | Capital structure is currently incomplete. |

| LC-GAP-041 | Business | Define taxes, depreciation, and accounting depth. | P1 | Major realism/scope choice. |

| LC-GAP-042 | Business | Define business failure, insolvency, closure, bankruptcy, and owner liability. | P1 | Completes downside risk. |

| LC-GAP-043 | Business | Define business sale valuation formula including assets, profit, contracts, reputation, debt, and capital capability. | P0 | Explicitly needed for sale realism. |

| LC-GAP-044 | Business | Define contract capacity reservation, deadlines, penalties, quality thresholds, and repeat-client logic. | P0 | Contracts are a major scaling loop. |

| LC-GAP-045 | Business | Define business reputation/brand/trust and how it affects demand and contracts. | P1 | Needed for non-price differentiation. |

| LC-GAP-046 | Economy | Define personal income tax, payroll tax, sales tax, property tax, and whether taxes are simplified or jurisdictional. | P0 | Affects all net-income targets. |

| LC-GAP-047 | Economy | Define inflation and cost-of-living model, if any. | P1 | Needed for multi-decade simulation balance. |

| LC-GAP-048 | Economy | Define banking products: checking, savings, interest, fees, credit cards, loans, mortgages. | P0 | Core personal finance system gap. |

| LC-GAP-049 | Economy | Define credit score/creditworthiness system. | P1 | Needed if loans and mortgages exist. |

| LC-GAP-050 | Economy | Define investment system scope: stocks, funds, bonds, retirement accounts, real estate, or none. | P0 | Major wealth-progression branch. |

| LC-GAP-051 | Economy | Define insolvency/personal bankruptcy and debt collection. | P2 | Downside completeness. |

| LC-GAP-052 | Economy | Define illicit-cash conversion/legitimization as a purely abstract game mechanic and its risk/cost rules. | P1 | Needed for underworld economy without procedural real-world detail. |

| LC-GAP-053 | Assets | Define housing tiers, rent/buy, mortgages, utilities, maintenance, and property appreciation. | P0 | Large missing life-sim system. |

| LC-GAP-054 | Assets | Define vehicle catalog, purchase/finance, insurance, fuel, maintenance, reliability, and depreciation. | P0 | Driving/logistics skills need asset context. |

| LC-GAP-055 | Assets | Define personal inventory scope versus abstract ownership records. | P1 | Controls UI complexity and market design. |

| LC-GAP-056 | Assets | Define item wear/durability for vehicles/equipment/assets. | P1 | Affects maintenance and replacement costs. |

| LC-GAP-057 | Crime | Finalize 23 abstract crime opportunity definitions, unlocks, payout bands, and risk bands. | P1 | Content taxonomy needs canonicalization. |

| LC-GAP-058 | Crime | Define Heat decay and escalation formula. | P0 | Core crime-risk loop. |

| LC-GAP-059 | Crime | Define detection/arrest probability formula and hard minimum risk. | P0 | Current principle exists; formula does not. |

| LC-GAP-060 | Crime | Define legal consequences in abstract terms: fines, temporary lockout/incarceration, record, career/business effects. | P0 | Crime needs meaningful downside. |

| LC-GAP-061 | Crime | Define criminal record persistence, expungement, and employer eligibility effects. | P1 | Connects underworld and legal career systems. |

| LC-GAP-062 | Crime | Define syndicate hierarchy, staffing, territory/market, and passive operations, all at abstract simulation level. | P1 | Needed for late-game criminal business path. |

| LC-GAP-063 | Relationships | Decide whether friendships, romance, marriage, children, and family are in scope. | P0 | Major life-sim identity decision. |

| LC-GAP-064 | Relationships | If relationships exist, define relationship meters, interaction costs, time requirements, and decay. | P1 | Core mechanics undefined. |

| LC-GAP-065 | Relationships | Define household finances and shared assets. | P1 | Connects relationships to economy. |

| LC-GAP-066 | Relationships | Define inheritance and heirs in the legacy system. | P1 | Potentially central to multi-generation play. |

| LC-GAP-067 | Health | Define healthcare system depth: insurance, doctors, prescriptions as abstract costs, emergencies, preventive care. | P0 | Healthcare skill/career already exists. |

| LC-GAP-068 | Health | Define health risk factors and recovery without turning the game into medical micromanagement. | P1 | Scope and balance decision. |

| LC-GAP-069 | Health | Define stress/burnout interaction with work hours, sleep, and performance. | P1 | Useful realistic constraint. |

| LC-GAP-070 | Events | Define random event engine categories, frequency, weighting, and cooldowns. | P1 | Adds variability to long progression. |

| LC-GAP-071 | Events | Define global economic events: recessions, booms, shortages, rate changes, etc. | P2 | Adds macroeconomic depth. |

| LC-GAP-072 | Reputation | Define separate personal, professional, business, and underworld reputation scores or a unified model. | P1 | Needed for offers, clients, and legal consequences. |

| LC-GAP-073 | Legacy | Define what causes a completed “character death” for Click Mastery and whether natural death and player-selected retirement both count. | P0 | Click Mastery depends on completed deaths. |

| LC-GAP-074 | Legacy | Define inheritance rules between characters and what persists besides Click Mastery. | P0 | Central to replay loop. |

| LC-GAP-075 | Legacy | Define whether legacy characters share a family/dynasty history screen. | P2 | Presentation/replay value. |

| LC-GAP-076 | UI | Finalize permanent HUD fields and responsive layouts for desktop/mobile. | P0 | Core navigation and readability. |

| LC-GAP-077 | UI | Define menu map and exact screen hierarchy. | P0 | Prevents feature sprawl and giant scroll pages. |

| LC-GAP-078 | UI | Define notification center, activity log retention, and alert severity. | P1 | Long-running systems need clear feedback. |

| LC-GAP-079 | UI | Define comparison UI for jobs, contracts, businesses, housing, vehicles, and financial products. | P1 | Reduces confusion, which is treated as a bug. |

| LC-GAP-080 | UI | Define tutorial/onboarding path and when advanced systems unlock. | P0 | Necessary for accessibility of a deep sim. |

| LC-GAP-081 | Technical | Finalize Rust Launcher 3.0 production acceptance and migration from Go 2.1.1. | P0 | Blocks final updater architecture. |

| LC-GAP-082 | Technical | Define canonical source-of-truth flow: game/index.html -> package -> stable manifest. | P0 | Prevents source/release drift. |

| LC-GAP-083 | Technical | Define permanent ID/versioning rules for every content type (skills, careers, upgrades, items, contracts, events). | P0 | Critical for save migration safety. |

| LC-GAP-084 | Technical | Define schema for feature flags and staged migrations. | P1 | Needed for safe long-lived saves. |

| LC-GAP-085 | Technical | Define automated compatibility test matrix across old save versions. | P0 | Save preservation is highest priority. |

| LC-GAP-086 | Technical | Define crash/error telemetry strategy, if any, with privacy boundaries. | P2 | Useful for debugging releases. |

| LC-GAP-087 | Balance | Define target time-to-first job, first $10k, first business, first $100k net worth, first $1M net worth, and late-game milestones. | P0 | Needed for progression pacing. |

| LC-GAP-088 | Balance | Define economy anti-exploit rules for rapid clicking, update timing, save reloads, and clock manipulation. | P1 | Protects balance in a time-based sim. |

| LC-GAP-089 | Accessibility | Define keyboard navigation, text scaling, reduced motion, color-blind support, and screen-reader baseline. | P1 | Should be designed before UI hardens. |

| LC-GAP-090 | Distribution | Define Windows packaging/install location, desktop shortcut, uninstall, and code-signing strategy. | P1 | Needed before broad distribution. |

| LC-GAP-091 | Distribution | Define monetization model: free, paid one-time, demo, supporter DLC, or other non-pay-to-win model. | P1 | Product strategy gap. |

| LC-GAP-092 | Content | Define content quantity targets per career branch, business industry, event category, contract class, and asset class for v1.0. | P0 | Turns “full game” into measurable scope. |

| LC-GAP-093 | Content | Define location/world model: one abstract region, selectable cities, or a geographic map. | P0 | Impacts costs, jobs, housing, businesses, and travel. |

| LC-GAP-094 | Content | Define travel/commute mechanics and how location affects available jobs/businesses. | P1 | Links driving, housing, careers, and time. |

| LC-GAP-095 | Debug | Define owner/developer console capabilities and guardrails for test saves. | P2 | Speeds QA without risking production saves. |

| LC-GAP-096 | Data | Define deterministic RNG/seeding rules for reproducible tests versus live randomness. | P1 | Important for QA and balance. |



# 28. Decision Worksheet Template

| Field | Entry |

| --- | --- |

| Gap ID | LC-GAP-___ |

| Decision |  |

| Chosen rule |  |

| Reasoning / design goal |  |

| Systems affected |  |

| Save migration needed? | Yes / No |

| Implementation task(s) |  |

| Test cases required |  |

| Status | OPEN / APPROVED / IMPLEMENTED / VERIFIED |

| Approved date |  |



## 28.1 Definition of “Closed”

# 29. Recommended Spec-Filling Workflow

## 29.1 Suggested First Decision Session

# Appendix A — Current Release Baseline

| Component | Baseline | Rule / note |

| --- | --- | --- |

| Game version | 7.0.7 | Current stable updater-validation release lineage. |

| Save version | 12 | Must remain compatible until a real schema change justifies a migration. |

| Stable launcher | Go 2.1.1 | Current stable lineage; had launcher-UI packaging/syntax problems. |

| Replacement launcher | Rust 3.0.0 candidate | In validation; should not be treated as stable until acceptance. |

| Production origin | http://127.0.0.1:8765 | Must remain stable to preserve browser-local saves. |

| Developer origin | http://127.0.0.1:8766 | Target isolated local-development origin. |

| Update feed | GitHub stable manifest | Package-first, manifest-last publication rule. |

| Current source issue resolved in candidate | Launcher UI must be readable source and JS-syntax validated | Avoid giant base64 UI embedded in CI YAML. |



# Appendix B — Canonical Business Upgrade Card

| Field | Requirement |

| --- | --- |

| Upgrade | Permanent ID + clear name |

| What it buys | Concrete capability/equipment/process change |

| Capital cost | Paid from company cash |

| Recurring cost | Monthly/periodic OPEX, if any |

| Skill gate | Relevant current skill vs required skill |

| Current → After | Exact measurable effect |

| Blocked because | Every unmet requirement listed |

| Details | Operational explanation and industry relevance |



# Appendix C — Canonical Career Offer Card

| Field | Requirement |

| --- | --- |

| Employer | Name, industry, size / archetype |

| Role | Permanent career role ID + title |

| Compensation | Base pay plus applicable overtime/bonus/commission/benefits |

| Schedule | Hours/days/remote/shift |

| Requirements | Education, credentials, training, experience, skills, eligibility |

| Career path | Immediate predecessor and possible next roles |

| Offer expiry | Explicit date/time or “no expiry” |

| Blocked because | Exact unmet condition |

| Actions | Accept / decline / inspect career tree |



# Appendix D — Release Safety Checklist

- Latest authoritative source confirmed.

- Working tree/release branch clean and identified.

- Version and save schema intentional.

- All persistent IDs checked for reuse/removal.

- Migration path present for every schema change.

- Old-save fixture tests pass.

- Game JavaScript syntax passes.

- Launcher JavaScript syntax passes.

- Native launcher formatting/tests/lint/build pass.

- Package deterministic; size and SHA-256 recorded.

- Bad package hash/size/version/JSON rejection tested.

- Backups and rollback/recovery tested.

- Windows release candidate tested from a clean extracted folder.

- Package uploaded before stable manifest.

- Published package re-downloaded/verified.

- Stable manifest updated last.

- Post-release updater detection/install test completed.


