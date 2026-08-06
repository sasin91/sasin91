# CV source — Jonas Hansen

Transcribed from `CV_Jonas_Hansen_Softwareengineer.pdf`, 2026-07-28.
This file is the authoritative record for `content/cv.toml`. It is not rendered.

**Jonas Hansen — Software developer**
Slagelse, 4200 · +45 50106917 · jonas.kerwin.hansen@gmail.com

## Intro

I have been working with PHP and Laravel since 2015 and since then I have helped
develop a ticketing agency, a video streaming platform and a recruitment platform.

I enjoy designing and delivering customized and solid solutions, but also enjoy
support and listening to user requests and issues with a smile.

<!-- The source PDF reads "takes" here. Jonas confirmed the intended sense is
     "enjoys", so the site renders "enjoy" to agree with the leading "I enjoy".
     Worth correcting in the source PDF too. -->

## Work history & achievements

### January 2017 – February 2020 — Lead developer, GHC Travel / Iraqi Airways, Copenhagen
Airline booking platform

- Joined as a junior developer maintaining a procedural PHP platform, then rewrote it to Laravel about a year in
- Grew from junior to senior developer; took over as sole developer after the senior developer left for a new job, continuing to build out the platform before leading development in the final phase
- Migrated from a traditional PHP web host to VPS servers I set up, managed and monitored, including DNS and Let's Encrypt SSL
- Maintained and upgraded the Laravel rewrite up to version 8, including a comprehensive PHPUnit test suite ensuring fast feedback loops on changes
- Debugged slow database queries and implemented caching to keep the platform responsive
- As the platform grew, domain boundaries became a mess — extracted the frontend into 3 distinct Nuxt 2 apps sharing UI and design packages over npm and consuming a Laravel API, establishing clear visual boundaries, and refactored the backend using domain-driven design
- Together with a new coworker, integrated and automated ticket booking and flight monitoring via UiPath RPA, improving ticket sales and customer satisfaction
- Provided live debugging and triage support directly to reception staff and agents, acting as first-line support

### April 2020 – January 2023 — Tech Lead, Syncronet, Slagelse
Live streaming social media platform

- Led migration from an expensive setup on Azure using media services to Linode with mux.com for video delivery
- Set up, managed and monitored VPS infrastructure, including DNS and Let's Encrypt SSL
- Architectured a Kubernetes cluster with Go services and Nginx endpoints wrapping FFmpeg for internal video processing and delivery
- Debugged slow database queries and implemented caching to keep the platform performant
- Led frontend development in Nuxt 3 and mobile development in React Native while maintaining Laravel backend

### January 2023 – August 2023 — Web developer, JUICE ApS, Copenhagen
Job & candidate matchmaking platform

- Introduced CI/CD enabling efficient and fast delivery of Symfony 6
- Contributed to multiple features in Symfony & Twig + Stimulus.js
- Worked in close collaboration with the CTO, leading development with his feedback and review

### September 2023 – September 2024 — Web developer, Supeo, Næstved
Web development agency

- Held primary responsibility for Laravel development, including sole ownership of customer interactions and development of SamFocus in Laravel 9
- Provided code review and development support across other work, including Supeo Flex in React, Express.js & GraphQL

### September 2024 – February 2026 — Web developer, JUICE ApS, Copenhagen
Job & candidate matchmaking platform

- Upgraded Symfony 6 to 7
- Built a comprehensive ranking and sorting engine, delivering a quick and efficient method of finding candidates and sorting by relevancy
- AI integration, making it a breeze to upload a job ad and receiving a SmartMatch job post
- Led development in close collaboration with the CTO, who provided feedback and code review

## Skills

- Linux server management and maintenance
- Web & App development
- Database administration

## Education

Two short-cycle higher educations.

### January 2012 – August 2013 — Strøm, styring & IT, Selandia CEU, Slagelse
Including Cisco CCNA and IP based network management.

### February 2014 – August 2015 — Web integrator, Roskilde Teknisk Skole, Høng

## Notes

- The source PDF spells Copenhagen as "Copenhangen" twice. Corrected above.
- Since February 2026: job-seeking. Not listed as a role.
- 2026-08-06: titles and achievements enriched with detail not in the source
  PDF, from Jonas directly — GHC Travel title changed to "Lead developer"
  (progression: junior → senior → sole developer → lead) and Syncronet title
  changed to "Software developer" → "Tech Lead". JUICE (both stints) and
  Supeo achievements expanded to describe close collaboration with the CTO
  and primary/review responsibilities. All roles were close collaborations
  on small to medium teams.
- 2026-08-06: GHC and Syncronet achievements expanded with infrastructure
  work (VPS setup, DNS, Let's Encrypt SSL, monitoring), database query
  debugging and caching, and — GHC only — live debugging/triage support for
  reception staff and agents. GHC's frontend bullet rewritten to describe
  extracting the Laravel/Vue 2 monolith into 3 Nuxt 2 apps sharing UI and
  design packages over npm against a Laravel API.
- 2026-08-06: GHC rewritten around the fuller timeline Jonas gave — joined
  into a procedural PHP app, rewrote it to Laravel about a year in, took
  over as sole developer when the senior left, and later (as the platform
  grew and domain boundaries got messy) extracted the Nuxt frontends to
  establish visual boundaries and refactored the backend with DDD; the RPA
  work covered ticket booking and flight monitoring, done with a new
  coworker. Jonas confirmed the company's 2020 COVID-19 closure should stay
  out of the public CV/summary and only be recorded here.
- Context (not in the public CV): GHC Travel / Iraqi Airways wound down in
  2020 due to COVID-19's impact on air travel — that is why the role ends
  February 2020.
