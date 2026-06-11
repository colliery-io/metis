---
id: metis-vision
level: vision
title: "Metis"
short_code: "METIS-V-0001"
status: published
created_at: 2025-07-05T11:02:47Z
updated_at: 2025-07-05T11:02:47Z
parent: 
blocked_by: 

# Phase progression for vision
tags:
  - "#vision"
  - "#phase/published"
  # - "#phase/review"
  # - "#phase/draft"

exit_criteria_met: true
---

# Metis Vision

## Purpose
Metis exists to solve the fundamental disconnect between software design and implementation. It eliminates the chaos of scattered documentation, forgotten decisions, and unclear work progression by providing an opinionated, hierarchical work management system that ensures every line of code traces back to a documented design decision. Metis is the honest backbone for a team's work across its fleet of repositories — a lightweight, self-hosted tracker with the daily-use core of tools like JIRA (work items, workflows, assignment, query) and none of their administrative sprawl. It serves humans and AI agents as equal first-class clients.

## Core Values
- **Design Before Code**: Every implementation must flow from documented design. This means rejecting the temptation to "just build it" and instead investing in clear thinking before clear coding.
- **Decisions Are Sacred**: Once made and documented, decisions become immutable records that can be superseded but never deleted. This means treating ADRs as first-class artifacts and preserving the historical "why" behind our choices even as they evolve.
- **Completion Over Deadlines**: Work progresses when it's ready, not when time expires. This means respecting the natural flow of understanding → design → implementation without artificial time pressure.
- **Progressive Clarity**: Documentation becomes more detailed as work progresses. This means avoiding premature specification while ensuring appropriate detail at each level.
- **Your Data, Your Infrastructure**: Self-hosted on open storage (PostgreSQL for teams, SQLite for solo use), with full markdown export at any time. This means rejecting SaaS lock-in and bespoke formats — teams own and control their work data, and can always walk away with readable files.

## Long-term Vision
 Teams using Metis will have one shared backbone of work spanning every repository they own: initiatives that cut across services, tasks that link and block each other regardless of where the code lives, and a unified view of what's in flight. They will have better recall of every architectural decision, seamless onboarding through self-documenting work history, and zero repeated discussions about past choices. Humans will use it through a clean UI; AI agents will use it through MCP from whichever repo they're working in — both reading and writing the same system of record. Metis will have proven that a team can run its entire work management on a single self-hosted binary and a database, without vendor lock-in or heavyweight tooling overhead.

## Success Definition
We'll know Metis has achieved its vision when:
- I can confidently demonstrate the methodology on my own projects with clear before/after examples
- Other developers who see Metis find it compelling enough to try on their projects
- The documentation and templates are clear enough that someone can adopt Metis without my direct help
- At least a few projects beyond my own have successfully used Metis and shared their experience
- The methodology proves valuable enough that I continue using it consistently on new projects

## Principles
- **Start Simple, Scale Smart**: Begin with markdown files and basic templates. Only add tooling and process as teams grow and need it. This guides us to resist over-engineering the solution.
- **Documentation Is Code**: Treat documentation with the same rigor as code - version controlled, reviewed, tested, and refactored. This guides us to build tooling that integrates with developer workflows.
- **Explicit Over Implicit**: Make phases, transitions, and relationships visible and required. This guides us to reject solutions that rely on convention or memory.
- **Developer Experience First**: If developers won't use it, it doesn't matter how good the theory is. This guides us to prioritize ergonomics and integration over features.
- **Learn From Reality**: Adapt based on how teams actually work, not how we think they should work. This guides us to gather feedback and iterate rather than prescribe perfection.

## Exit Criteria
- [x] Purpose is clear and resonates with all stakeholders
- [x] Core values are defined and actionable
- [x] Long-term vision is inspiring and achievable
- [x] Success definition is measurable
- [x] Principles provide clear guidance for decisions