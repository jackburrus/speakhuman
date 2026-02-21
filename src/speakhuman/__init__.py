"""Main package for speakhuman."""

from __future__ import annotations

from speakhuman.filesize import naturalsize
from speakhuman.i18n import activate, deactivate, decimal_separator, thousands_separator
from speakhuman.lists import natural_list
from speakhuman.number import (
    apnumber,
    clamp,
    fractional,
    intcomma,
    intword,
    metric,
    ordinal,
    scientific,
)
from speakhuman.time import (
    naturaldate,
    naturalday,
    naturaldelta,
    naturaltime,
    precisedelta,
)

from importlib.metadata import version

__version__ = version("speakhuman")

__all__ = [
    "__version__",
    "activate",
    "apnumber",
    "clamp",
    "deactivate",
    "decimal_separator",
    "fractional",
    "intcomma",
    "intword",
    "metric",
    "natural_list",
    "naturaldate",
    "naturalday",
    "naturaldelta",
    "naturalsize",
    "naturaltime",
    "ordinal",
    "precisedelta",
    "scientific",
    "thousands_separator",
]
