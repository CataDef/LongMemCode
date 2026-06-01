#!/usr/bin/env python3
"""Generate clean comparison visuals for the SWE-bench Verified result."""
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import FancyBboxPatch

OUT = "/Users/catalinjibleanu/LongMemCode/swebench_verified_argos"

# ---- palette ----
INK = "#0B1221"; SUB = "#5B6472"; GRID = "#E6E9EF"
VANILLA = "#9AA4B2"; ARGOS = "#2E6BFF"; OFFICIAL = "#C9D2E0"; ACCENT = "#10B981"

# ============== FIGURE 1: headline bar ==============
fig, ax = plt.subplots(figsize=(9, 5.2), dpi=200)
fig.patch.set_facecolor("white"); ax.set_facecolor("white")

labels = ["Opus 4.8\n(on its own)", "Anthropic\npublished", "Opus 4.8\n+ ArgosBrain"]
vals   = [87.0, 88.6, 91.4]
colors = [VANILLA, OFFICIAL, ARGOS]
x = range(len(vals))
bars = ax.bar(x, vals, width=0.62, color=colors, zorder=3,
              edgecolor="white", linewidth=1.5)

for i, (b, v) in enumerate(zip(bars, vals)):
    ax.text(b.get_x()+b.get_width()/2, v+0.35, f"{v:.1f}%",
            ha="center", va="bottom", fontsize=20, fontweight="bold",
            color=ARGOS if i == 2 else INK)

# lift annotation
ax.annotate("", xy=(2, 91.4), xytext=(0, 87.0),
            arrowprops=dict(arrowstyle="-|>", color=ACCENT, lw=2.2,
                            connectionstyle="arc3,rad=-0.25"))
ax.text(1.18, 90.6, "+4.4 pts\n22 tasks rescued", color=ACCENT,
        fontsize=12.5, fontweight="bold", ha="left", va="center")

ax.set_ylim(84, 93)
ax.set_xticks(list(x)); ax.set_xticklabels(labels, fontsize=12.5, color=INK)
ax.set_yticks([84,86,88,90,92])
ax.set_yticklabels([f"{t}%" for t in [84,86,88,90,92]], fontsize=10, color=SUB)
ax.yaxis.grid(True, color=GRID, lw=1, zorder=0); ax.set_axisbisect = None
for s in ["top","right","left"]: ax.spines[s].set_visible(False)
ax.spines["bottom"].set_color(GRID)
ax.tick_params(length=0)

ax.set_title("SWE-bench Verified  ·  Claude Opus 4.8",
             fontsize=17, fontweight="bold", color=INK, pad=16, loc="left")
ax.text(0, 1.005, "", transform=ax.transAxes)
fig.text(0.125, 0.915, "A retrieval engine that looks up the codebase lifts a frontier model past its own ceiling",
         fontsize=11, color=SUB)
fig.text(0.125, 0.02, "github.com/CataDef/LongMemCode/tree/main/swebench_verified_argos",
         fontsize=9.5, color=ARGOS, fontweight="bold")
plt.tight_layout(rect=[0,0.03,1,0.90])
plt.savefig(f"{OUT}/headline.png", facecolor="white", bbox_inches="tight")
print("wrote headline.png")

# ============== FIGURE 2: rescue breakdown table-ish ==============
fig2, ax2 = plt.subplots(figsize=(9, 5.4), dpi=200)
fig2.patch.set_facecolor("white"); ax2.set_facecolor("white"); ax2.axis("off")

repos = [("django",7),("sphinx",4),("sympy",4),("astropy",2),
         ("matplotlib",2),("xarray",2),("pytest",1)]
ax2.text(0.04, 0.93, "22 tasks Opus 4.8 failed alone — solved with ArgosBrain",
         fontsize=15.5, fontweight="bold", color=INK, transform=ax2.transAxes)
ax2.text(0.04, 0.86, "by project", fontsize=11, color=SUB, transform=ax2.transAxes)

y = 0.76; maxv = 7
for name, n in repos:
    ax2.text(0.04, y, name, fontsize=12.5, color=INK, va="center", transform=ax2.transAxes)
    barw = 0.46 * (n/maxv)
    ax2.add_patch(FancyBboxPatch((0.30, y-0.022), barw, 0.044,
                  boxstyle="round,pad=0.002,rounding_size=0.01",
                  fc=ARGOS, ec="none", transform=ax2.transAxes, zorder=3))
    ax2.text(0.30+barw+0.012, y, str(n), fontsize=12, fontweight="bold",
             color=ARGOS, va="center", transform=ax2.transAxes)
    y -= 0.095

ax2.text(0.04, 0.04, "Every fix used Argos to look up the code first  ·  full data + patches in the repo",
         fontsize=9.5, color=SUB, transform=ax2.transAxes)
plt.savefig(f"{OUT}/rescues_by_repo.png", facecolor="white", bbox_inches="tight")
print("wrote rescues_by_repo.png")
