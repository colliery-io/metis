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
Metis exists to solve the fundamental disconnect between software design and implementation at the project level. It eliminates the chaos of scattered documentation, forgotten decisions, and unclear work progression by providing an opinionated, hierarchical documentation system that ensures every line of code traces back to a documented design decision. Metis focuses on project-level planning and decision management, not enterprise cross-team dependency coordination.

## Core Values
- **Design Before Code**: Every implementation must flow from documented design. This means rejecting the temptation to "just build it" and instead investing in clear thinking before clear coding.
- **Decisions Are Sacred**: Once made and documented, decisions become immutable records that can be superseded but never deleted. This means treating ADRs as first-class artifacts and preserving the historical "why" behind our choices even as they evolve.
- **Completion Over Deadlines**: Work progresses when it's ready, not when time expires. This means respecting the natural flow of understanding → design → implementation without artificial time pressure.
- **Progressive Clarity**: Documentation becomes more detailed as work progresses. This means avoiding premature specification while ensuring appropriate detail at each level.
- **Open Formats Over Vendor Lock-in**: Use standard formats like markdown and avoid proprietary systems. This means rejecting large centralized tools with bespoke formats in favor of portable, readable files that teams own and control.

## Long-term Vision
 Teams using Metis will have better recall of every architectural decision, seamless onboarding of new developers through self-documenting codebases, and zero repeated discussions about past choices. The MCP-powered tooling will feel invisible, with documentation naturally emerging from the development process rather than being a separate burden. Metis will have proven that teams can maintain high-quality documentation without vendor lock-in or heavyweight tooling overhead.

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