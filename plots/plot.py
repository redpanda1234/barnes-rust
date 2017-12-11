import numpy as np
import pandas as pd
import math

from matplotlib import pyplot as plt

import matplotlib.lines as mlines
import matplotlib.patches as mpatches

def add_arrow_to_line2D(
    axes, line, arrow_locs=[0.2, 0.4, 0.6, 0.8],
    arrowstyle='-|>', arrowsize=1, transform=None):
    """
    Add arrows to a matplotlib.lines.Line2D at selected locations.

    Parameters:
    -----------
    axes:
    line: Line2D object as returned by plot command
    arrow_locs: list of locations where to insert arrows, % of total length
    arrowstyle: style of the arrow
    arrowsize: size of the arrow
    transform: a matplotlib transform instance, default to data coordinates

    Returns:
    --------
    arrows: list of arrows
    """
    if not isinstance(line, mlines.Line2D):
        raise ValueError("expected a matplotlib.lines.Line2D object")
    x, y = line.get_xdata(), line.get_ydata()

    arrow_kw = {
        "arrowstyle": arrowstyle,
        "mutation_scale": 10 * arrowsize,
    }

    color = line.get_color()
    use_multicolor_lines = isinstance(color, np.ndarray)
    if use_multicolor_lines:
        raise NotImplementedError("multicolor lines not supported")
    else:
        arrow_kw['color'] = color

    linewidth = .25*line.get_linewidth()
    if isinstance(linewidth, np.ndarray):
        raise NotImplementedError("multiwidth lines not supported")
    else:
        arrow_kw['linewidth'] = linewidth

    if transform is None:
        transform = axes.transData

    arrows = []
    for loc in arrow_locs:
        s = np.cumsum(np.sqrt(np.diff(x) ** 2 + np.diff(y) ** 2))
        n = np.searchsorted(s, s[-1] * loc)
        arrow_tail = (x[n], y[n])
        arrow_head = (np.mean(x[n:n + 2]), np.mean(y[n:n + 2]))
        p = mpatches.FancyArrowPatch(
            arrow_tail, arrow_head, transform=transform,
            **arrow_kw)
        axes.add_patch(p)
        arrows.append(p)
    return arrows

def p3():

    # df1 = pd.read_csv("output_plot.csv")
    # y1 = df1["v_mag"]
    # x1 = df1["p_mag"]

    # df2 = pd.read_csv("output_plot_2.csv")
    df2 = pd.read_csv("binary.csv")
    y2 = df2["v_mag"]
    x2 = df2["p_mag"]

    f, ax = plt.subplots(1, 1)

    # line1, = ax.plot(x1, y1, 'k-')

    line2, = ax.plot(x2, y2, "k-")

    # add_arrow_to_line2D(ax, line1, arrow_locs=np.linspace(0., 1., 200),
    #                 arrowstyle='->')

    add_arrow_to_line2D(ax, line2, arrow_locs=np.linspace(0., 1., 200),
                    arrowstyle='->')

    ax.set_title('Phase space plot of position vs velocity for a star in a binary system')
    ax.set_xlabel('velocity')
    ax.set_ylabel('position')

    plt.show()



def p4():

    df1 = pd.read_csv("binary.csv")
    y1 = df1["y"]
    x1 = df1["x"]

    # df2 = pd.read_csv("output_plot_2.csv")
    y2 = df1["vy"]
    x2 = df1["vx"]

    f, (ax1, ax2) = plt.subplots(1, 2)

    line1, = ax1.plot(x1, y1, 'k-')

    line2, = ax2.plot(x2, y2, 'k-')

    ax1.set_title('Configuration space plot of x vs y')
    ax1.set_xlabel('x')
    ax1.set_ylabel('y')

    ax2.set_title('State space plot of vx vs vy')
    ax2.set_xlabel('vx')
    ax2.set_ylabel('vy')

    add_arrow_to_line2D(ax1, line1, arrow_locs=np.linspace(0., 1., 200),
                     arrowstyle='->')

    add_arrow_to_line2D(ax2, line2, arrow_locs=np.linspace(0., 1., 200),
                    arrowstyle='->')

    plt.show()
