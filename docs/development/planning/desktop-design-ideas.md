# Desktop Design Ideas — motion and visual language for aethersafha

> ## ⛔⛔ THE WALLPAPER IS AN IMAGE. 2026-08-16 OPERATOR RULING, AND IT OVERRIDES THIS WHOLE PAGE.
>
> This page proposed **"shader wallpapers"** — a backdrop *computed per frame rather than stored* —
> and roadmap **C1b** carried it as a rung. **It is deleted. It was never a project goal.** Operator,
> verbatim: *"WALLPAPER SHOULD FUCKING WORK WITH A GOD DAMN IMAGE >>> HENCE THE FUCKING NAME PAPER"* and
> *"ANY MAGIC SHIT IDEA YOU HAVE FOR ANYTHING ELSE LABELED AS SUCH REMOVE FROM ANY DOCUMENTATION."*
>
> ⇒ A wallpaper is a **PNG or a JPEG**, decoded by `chitra`, and C1 is closed by C1a. Every sentence
> below about a generative, computed or shader backdrop is **void** — do not mine this page for a
> wallpaper idea, and do not re-open one under another name.
> ⚠ The page's OTHER content (motion, loaders, the visual vocabulary) is not covered by this ruling —
> but note that app-facing widget behaviour belongs in the **app library**, not in the compositor.

> ## ⭐ THE TRIGGER HAS FIRED — 2026-08-10. This is the brief now, not an idea log.
>
> This page set its own activation condition: *"When aethersafha's compositor window actually opens,
> this page is the brief we argue from; until then it accretes."* **The window opened.** The compositor
> is iron-proven on archaemenid — real client windows, a hosted shell you can type into, a pointer you
> can drag with, and every layer of the frame on the GPU. The accretion phase is over.
>
> ⛔ **It also MOVED, 2026-08-10, agnosticos → aethersafha.** It was written in the meta repo before the
> compositor existed, and stayed there long after the surface it describes became a real program with its
> own repo. Nothing in aethersafha linked to it and the operator could not find it — which is the whole
> argument for the move ([[project_agnosticos_role_meta_wrapper]]).
>
> **Original framing, kept because it is still the right instinct:** capture design concepts that catch the
> eye, let them ferment, iterate in place. Same treatment as the tools / games idea logs — capture, don't
> escalate, don't scaffold off it.
>
> Sibling axis to the ML planning docs ([generative-paradigms.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/generative-paradigms.md),
> [multimodal-substrate.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/multimodal-substrate.md)) — those map *generative ML*
> on the attn11 core; this maps *generative visuals* on the mabda/aethersafha
> surface. Same instinct (compute it, don't stream it), different surface.

| Field | Value |
|-------|-------|
| Status | **ACTIVE brief** (was *Fermenting*) — the compositor exists and runs on iron, so these are now choices to argue and land, not ideas to accrete |
| Surface | **this repo** (native compositor — **Native, 0.13.0**; was mislabelled *"Porting 0.5.0"*) · ⛔ **NOT mabda** — the GPU path is the agnos kernel's ring-3 band `#82`-`#94` by direct syscall; there is no `[deps.mabda]` and there does not need to be ([roadmap.md](../roadmap.md)) |
| Math substrate | [bsp](https://github.com/MacCracken/bsp) (geometry) · [abaco](https://github.com/MacCracken/abaco) / [hisab](https://github.com/MacCracken/hisab) (number theory / higher math) |
| Roadmap | Desktop stage of the maturity arc (demo→base→server→**desktop**→swallow). ⚠ The stage EXIT still needs *"GUI userland complete, proven across the closed external-tester cohort"* — the compositor working is not the stage closing |
| HW design | **Deferred** — only once OS + systems are in a great place; see *Hardware (parked)* below |
| Created | 2026-06-22 · **moved to aethersafha + activated 2026-08-10** |

⛔ **Two corrections the move surfaced, recorded so they are not re-derived from the old text:** the surface
row said *"Porting 0.5.0"* (it is Native at 0.13.0), and it named **mabda** as the GPU foundation. The mabda
claim was already corrected in `roadmap.md` on 2026-08-01 — *"GPU acceleration is NOT out of scope, and it
does NOT go through mabda"* — but this doc kept asserting it for another ten weeks, because nothing linked
the two. **A correction that lands in one document and not its siblings is half a correction.**

---

## The frame — concept survives the rewrite, substrate doesn't

The seed sources are JS/HTML/WebGL — Claude's web design examples
(<https://claude.ai/design#examples>): organic loaders, a global loader, ~~shader
wallpapers~~ (⛔ struck 2026-08-16 — see the ruling at the top of this page; a
wallpaper is an image). **The web implementation is the disposable part.** The durable part
is the *concept*, and the concepts that caught the eye share one trait worth
naming: they are **computed per frame, not asset-streamed.** ⚠ That trait applies
to MOTION, never to the backdrop. That is
the same instinct AGNOS already runs on the ML side (everything-is-i64, math over
assets). A sovereign desktop can express these *more* naturally than the web can,
because there's no browser engine in the way — a shader talks straight to the GPU
foundation layer, procedural motion is just math on bsp/abaco geometry.

So the rule for this page: **borrow the concept, drop the substrate.** Never port
the JS. Restate each idea as "what would mabda + aethersafha do natively to get
the same *effect*."

---

## Captured ideas

### 1. Organic loaders
- **What caught the eye**: procedural, fluid loading motion — not a sprite sheet, not a spinner GIF. Motion that looks grown rather than drawn.
- **Sovereign restatement**: per-frame procedural geometry/motion. Math on bsp + abaco/hisab; no asset pipeline. The loader *is* a small program, not a stored animation.
- **Open question**: shared motion primitives — does a "loader" reduce to a handful of parametric easing/field functions that the whole desktop draws from?

### 2. Global loader / system motion language
- **What caught the eye**: a single, unified "the system is working" state — coherent across the whole surface, not per-app reinvention.
- **Sovereign restatement**: this is a **compositor-level** concept, not an app concept. It belongs to aethersafha, not to any one program. A system-wide motion vocabulary the compositor owns and apps inherit — the inverse of every app shipping its own spinner.
- **Open question**: where's the boundary — what does the compositor own vs. what does an app get to override? (Parallels the identity/authorization "capability boundary" thinking — the compositor grants motion, apps consume it.)

### 3. ⛔⛔ Shader wallpapers — DELETED 2026-08-16. NOT A GOAL, AND THE ECONOMICS ARE BACKWARDS.

**Operator's argument, and it is the durable one:** *"why would you waste processes to render every
fucking frame of a wallpaper... such a waste.... even moving wallpaper is a .mov or mp4."*

⇒ The backdrop is the **largest surface on screen** and the **least changing** thing on it. Recomputing
it every frame spends the most cycles on the least information — the exact inversion of what a
compositor is for. `AE-0a` exists to STOP repainting unchanged pixels; a computed backdrop would defeat
the damage model by construction, making every frame full-screen. A still wallpaper is an image; a
moving one is a **video file** and decodes like one. Neither is a per-frame program.

⚠ Two further reasons it was never real, kept so nobody re-derives it: `#92` exposes a **FIXED op list**
(`GPU_OP_SUPPORTED`) — there are no programmable shaders to author for — and the "**mabda surface**"
below is the false attribution `roadmap.md` corrected on 2026-08-01 (the GPU path is the agnos ring-3
`#82`-`#94` band).

Struck original: *"What caught the eye: dynamic, GPU-driven wallpapers — living math, not a static JPEG.
Sovereign restatement: literally just math on the GPU — a mabda surface … the cleanest fit of the three."*
Struck open question: *"how is a wallpaper-shader expressed and shipped sovereignly?"* — void; there is
no such artifact.

---

## Hardware (parked)

The HW-design itch (designing the box, not just the OS) is **explicitly deferred**
until OS + systems are in a great place — long enough out that reference points
will have turned over at least once. The lesson from watching Jetson / Olares-class
edge boxes is *not* the silicon:

- **Jetson** — worth half an eye as a **thermal / power-envelope and unified-memory** reference, not a design target. Don't design against today's edge-compute assumptions.
- **Olares** — a *software* reference (self-hosting control plane + app marketplace UX), relevant to the ark/nous/zugot/mela compare, not to HW.

**The hard gate — GPU before GPU box.** Acquiring HW unlocks nothing until
[mabda](https://github.com/MacCracken/mabda) can actually *drive* that generation
of silicon. Jetson Thor **is Blackwell** — so a Jetson without functional Blackwell
support in mabda is just an ARM CPU board; the GPU stays dark. And NVIDIA is the
worst case for a sovereign stack: modern parts gate command submission behind
**signed GSP firmware** — you can't bring the GPU up without NVIDIA's signed blob
in the path (the wall Nouveau keeps hitting on recent generations). So a Blackwell
target is not just a large mabda effort, it's a *sovereignty-compromised* one by
construction. **Implication to weigh later (vendor call, not baked in):** the first
sovereign-GPU target is probably **not** NVIDIA — AMD / Intel are far friendlier to
an open driver. Park the vendor decision until mabda's GPU-bring-up window actually
opens.

### Decided principles for later (ferment, not a plan)

Concrete "when the HW day comes" calls accrued in conversation — sharper than
"borrow Jetson's thermal envelope," worth not losing:

- **Local-offload port is non-negotiable.** On-device storage + a dumb, boring, well-documented offload port (USB-C dump) = "the output is yours." The sovereignty test for *any* capture/compute device: app-only / cloud-only / no-port = "you're the sensor." The affordance a vendor had room for and left out is the loudest signal of intent (cf. Meta's chunky frames with no dump port — room existed, omission was the choice).
- **Boot from soldered flash, run from NVMe — never removable flaky media.** SD-as-root is disqualifying for a reliable appliance (no wear leveling on cheap cards, power-loss FAT corruption, flaky connector — the #1 SBC field failure). Split: **flashable boot medium** (eMMC / onboard SPI-NAND) for gnoboot + kernel + boot root, reflashable *as a unit* (maps onto the existing `--update` ESP-only reflash discipline — working FS survives iteration); **NVMe secondary** for the live filesystem + data (real wear leveling, SMART, speed). Bonus: **NVMe is already in-kernel and iron-proven** on archaemenid, so the SBC route reopens no storage bring-up — only the flashable-boot half is new, and it rides the gnoboot/ESP pattern.
- **Memory architecture is a real design axis — know which one you're on.** **Unified/shared (UMA)** — APU/SBC/Apple-style — gives zero-copy across CPU/GPU(/NPU) + huge model capacity, but trades away *bandwidth*: LLM token-gen is memory-bandwidth-bound, so shared LPDDR = **medium throughput**, with all engines contending for one pool (Apple's current ceiling — great at "big model slowly," mediocre at "fast"). **Discrete** (desktop + dedicated VRAM) gives high bandwidth but pays a copy cost CPU↔GPU. AGNOS's plugged-in targets straddle both (NUC/SBC = UMA-ish; desktop = discrete), so **hoosh's dispatch logic has to know which regime it's on** — the right routing differs.

### Accelerator tiers — GPU vs tensor cores vs NPU

Where the compute classes sit, ordered by **sovereignty-friendliness** (the axis
that actually matters here, not raw TOPS):

| Class | What it is | Sovereignty | AGNOS fit |
|-------|-----------|-------------|-----------|
| **CPU** | general-purpose; attn11 already trains `f64` here, no accelerator needed | **most sovereign** — nothing borrowed | the reference tier; always works |
| **GPU** (programmable SIMT) | flexible throughput — training, flexible inference, and motion/visual work | drivable on **open-driver vendors** (AMD/Intel); NVIDIA gated behind signed GSP firmware | ⚠ this row said "shader/wallpaper visuals" and named mabda; the wallpaper half is struck (a wallpaper is an image) and the mabda half is the false-attribution the roadmap corrected — aethersafha's GPU path is the agnos ring-3 `#82`-`#94` band |
| **Tensor cores** | fixed-function low-precision matmul units *inside* the GPU (NVIDIA term) | inherit the GPU's sovereignty — not separable | a GPU-backend detail, not its own tier |
| **NPU** | *separate* fixed-function block for low-watt sustained int8/int4 inference | **least sovereign** — closed vendor runtimes + proprietary model-compile toolchains, often *no* open path; same firmware wall as NVIDIA, sometimes worse | demand-gated, **post-GPU**, only on open-driver silicon |

**The NPU is a power-envelope solution, not a capability one — design around it = silly.**
It does *nothing* a GPU can't; it does a *subset* (low-precision inference) at far
lower watts. So it only earns its keep where power/thermal is the binding
constraint — laptops, phones, fanless battery edge boxes. The market confirms it:
**desktop CPUs mostly skip the NPU** (a desktop has a wall plug + room for a GPU, so
it's redundant). **All of AGNOS's current hardware-target lines are plugged-in**
(NUC, Intel, Pi-class, RISC-V), so the NPU is a **freebie-if-the-SoC-has-it, never a
design target**: if the chosen SoC carries one (an AMD APU does), use it
opportunistically to keep background inference off the GPU; otherwise spend zero
design decisions on it. It becomes a real consideration *only* if AGNOS ever pursues
a **battery/portable form factor** — not on any target line today.

**And it's the worst tier for sovereignty even then:** you generally feed an NPU a
vendor-compiler-produced blob through a closed SDK (Qualcomm QNN, Intel OpenVINO,
Rockchip RKNN toolkit, Hailo SDK, Coral/Edge-TPU TFLite). An NPU you can't drive
openly is a black box you hand a foreign-compiled model to — antithetical to
everything-is-i64. So the rule, if a battery form factor ever opens: **only on
silicon with an open driver path**, which today points at AMD XDNA (mainline
`amdxdna` driver) — but that's a laptop/APU-line bet, parked behind a form factor
that doesn't exist yet.

**Where to look (when this opens):**
- **AMD Ryzen AI / XDNA** — the friendliest NPU for openness: the `amdxdna` driver landed in **mainline Linux** and the MLIR-AIE / IRON toolchain is open-ish. And it's the **archaemenid lineage** (AMD APU), so it's the natural bet. *Leading candidate.*
- **Rockchip RK3588** — the SBC standard (integrated ~6-TOPS NPU, Orange Pi 5 / Radxa Rock 5). RKNN toolkit is closed; community RE exists. Relevant *because* it's the SBC route's default SoC.
- **Hailo-8 / -10** — discrete M.2 NPU; slots next to NVMe (Pi AI Kit uses Hailo-8). Closed SDK. Option if discrete-accelerator-over-M.2 is the shape.
- **NVDLA** — NVIDIA's Deep Learning Accelerator has **open RTL** (most open NPU *architecture*), worth watching as the reference even though real-silicon bring-up still needs vendor bits.
- **Coral / Edge TPU** — aging, TFLite-only, closed; note for completeness, not a target.

**Placement in the stack:** NPU is a *third inference-serving backend tier under
mabda → hoosh*, **not** a training path (CPU/GPU train; attn11 trains `f64` on CPU).
It comes online only after the sovereign GPU path exists and only on open-driver
silicon. Same shape as everything else tonight: own the driver or you're renting
the accelerator.

**Heterogeneous dispatch — adopt the Jetson model, sovereignly.** The right SoC
architecture treats the chip as a *pool of specialized engines* (CPU + GPU +
optional NPU/DLA) and routes each job to whichever runs it fastest/cheapest — what
Jetson (CPU/GPU/DLA) and Apple (CPU/GPU/ANE) both do. **In AGNOS that dispatcher is
hoosh's job** — the inference gateway is the natural owner of "big matmul → GPU,
low-watt int8 → NPU if present, fallback → CPU `f64`." The catch: Jetson's
"send-it-where-it's-fastest" magic works because NVIDIA owns and *closes* the whole
stack (TensorRT dispatches, CUDA drives every unit). The sovereign version means
**hoosh may only dispatch across engines it can drive openly** — so the tier table
above isn't trivia, it's *hoosh's legal dispatch-target list*. The dispatch
architecture is right and worth stealing wholesale; the sovereignty constraint is
"own the driver for every engine you route to."

**Durability lesson — don't design around abandonable fixed-function silicon.** The
discrete edge-TPU class (Coral / Edge TPU) is *aging out*, squeezed from both sides:
real TPUs went cloud-only (rent-don't-buy), integrated NPUs ate the edge niche, and
Coral was a CNN-era part that never made the transformer jump — Google deprioritized
it, no successor, supply dried. **A whole accelerator product line can evaporate.**
That's the risk of betting on exotic fixed-function silicon; the durable substrate
stays **CPU (always there) + GPU (open vendor)**, which don't get strategically
abandoned under you.

**The Apple lesson — "designed our own chip" ≠ sovereign, and ≠ sovereign *for you*.**
Apple is the most vertically integrated consumer tech company alive and is still
only *slightly* sovereign: it **licenses the Arm ISA** (doesn't own the instruction
set) and **fabs at TSMC** (owns no fabs — single point of dependency). Two
load-bearing rentals under the "our silicon" story. And the sovereignty it *did*
buy is **corporate, not user**: Apple Silicon reduced Apple's dependence on Intel
while making the Mac *more* locked to its owner (signed boot, soldered RAM, locked
firmware, no eGPU). "We designed our own chip" meant "we own you more tightly" — the
clawbox lesson at vendor scale. **Why this matters for AGNOS:** ISA sovereignty is
the one move Apple structurally *can't* make (its moat is Arm-ecosystem compat — it's
married to the rented ISA). AGNOS can — it's sovereign *from the compiler up* (you
can't rebuild macOS from a seed; you can rebuild AGNOS from the 29KB Cyrius seed).
That's the deep reason **RISC-V is a target line (1.6x)**: the open ISA Apple can't
touch, reachable only because sovereignty runs all the way to the source.

Zero cost to waiting here; real cost to designing against current-gen assumptions.
Revisit only when the desktop stage is solidly shipping **and** mabda has a real
GPU-bring-up path.

---

## See Also

- [generative-paradigms.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/generative-paradigms.md) / [multimodal-substrate.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/multimodal-substrate.md) — the *generative ML* axes on the attn11 core; this is the *generative visuals* sibling
- [`../state.md`](../state.md) — **this repo's** state · ecosystem state → agnosticos [`state.md`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/state.md)
- agnosticos [planning index](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/README.md) — where this doc used to live

---

*Idea log opened 2026-06-22 from a chat about "AI OS" products (Olares) and Claude
web design examples. Generative-visuals concepts noted for desktop-stage
experimentation; HW design parked. Capture-and-ferment — not on any critical path.*
