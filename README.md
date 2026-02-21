# speakhuman

Rust-accelerated fork of [humanize](https://github.com/python-humanize/humanize).

[![MIT License](https://img.shields.io/github/license/jackburrus/humanize.svg)](LICENCE)

This modest package contains various common humanization utilities, like turning a
number into a fuzzy human-readable duration ("3 minutes ago") or into a human-readable
size or throughput. It is localized to:

- Arabic
- Basque
- Bengali
- Brazilian Portuguese
- Catalan
- Danish
- Dutch
- Esperanto
- European Portuguese
- Finnish
- French
- German
- Greek
- Hebrew
- Indonesian
- Italian
- Japanese
- Klingon
- Korean
- Norwegian
- Persian
- Polish
- Russian
- Simplified Chinese
- Slovak
- Slovenian
- Spanish
- Swedish
- Turkish
- Ukrainian
- Uzbek
- Vietnamese

<!-- usage-start -->

## Installation

### From PyPI

```bash
python3 -m pip install --upgrade speakhuman
```

### From source

```bash
git clone https://github.com/jackburrus/humanize
cd speakhuman
python3 -m pip install -e .
```

## Usage

### Integer humanization

```pycon
>>> import speakhuman
>>> speakhuman.intcomma(12345)
'12,345'
>>> speakhuman.intword(123455913)
'123.5 million'
>>> speakhuman.intword(12345591313)
'12.3 billion'
>>> speakhuman.apnumber(4)
'four'
>>> speakhuman.apnumber(41)
'41'
```

### Date & time humanization

```pycon
>>> import speakhuman
>>> import datetime as dt
>>> speakhuman.naturalday(dt.datetime.now())
'today'
>>> speakhuman.naturaldelta(dt.timedelta(seconds=1001))
'16 minutes'
>>> speakhuman.naturalday(dt.datetime.now() - dt.timedelta(days=1))
'yesterday'
>>> speakhuman.naturalday(dt.date(2007, 6, 5))
'Jun 05'
>>> speakhuman.naturaldate(dt.date(2007, 6, 5))
'Jun 05 2007'
>>> speakhuman.naturaltime(dt.datetime.now() - dt.timedelta(seconds=1))
'a second ago'
>>> speakhuman.naturaltime(dt.datetime.now() - dt.timedelta(seconds=3600))
'an hour ago'
```

### Precise time delta

```pycon
>>> import speakhuman
>>> import datetime as dt
>>> delta = dt.timedelta(seconds=3633, days=2, microseconds=123000)
>>> speakhuman.precisedelta(delta)
'2 days, 1 hour and 33.12 seconds'
>>> speakhuman.precisedelta(delta, minimum_unit="microseconds")
'2 days, 1 hour, 33 seconds and 123 milliseconds'
>>> speakhuman.precisedelta(delta, suppress=["days"], format="%0.4f")
'49 hours and 33.1230 seconds'
```

#### Smaller units

If seconds are too large, set `minimum_unit` to milliseconds or microseconds:

```pycon
>>> import speakhuman
>>> import datetime as dt
>>> speakhuman.naturaldelta(dt.timedelta(seconds=2))
'2 seconds'
```

```pycon
>>> delta = dt.timedelta(milliseconds=4)
>>> speakhuman.naturaldelta(delta)
'a moment'
>>> speakhuman.naturaldelta(delta, minimum_unit="milliseconds")
'4 milliseconds'
>>> speakhuman.naturaldelta(delta, minimum_unit="microseconds")
'4 milliseconds'
```

```pycon
>>> speakhuman.naturaltime(delta)
'now'
>>> speakhuman.naturaltime(delta, minimum_unit="milliseconds")
'4 milliseconds ago'
>>> speakhuman.naturaltime(delta, minimum_unit="microseconds")
'4 milliseconds ago'
```

### File size humanization

```pycon
>>> import speakhuman
>>> speakhuman.naturalsize(1_000_000)
'1.0 MB'
>>> speakhuman.naturalsize(1_000_000, binary=True)
'976.6 KiB'
>>> speakhuman.naturalsize(1_000_000, gnu=True)
'976.6K'
```

### Human-readable floating point numbers

```pycon
>>> import speakhuman
>>> speakhuman.fractional(1/3)
'1/3'
>>> speakhuman.fractional(1.5)
'1 1/2'
>>> speakhuman.fractional(0.3)
'3/10'
>>> speakhuman.fractional(0.333)
'333/1000'
>>> speakhuman.fractional(1)
'1'
```

### Scientific notation

```pycon
>>> import speakhuman
>>> speakhuman.scientific(0.3)
'3.00 x 10⁻¹'
>>> speakhuman.scientific(500)
'5.00 x 10²'
>>> speakhuman.scientific("20000")
'2.00 x 10⁴'
>>> speakhuman.scientific(1**10)
'1.00 x 10⁰'
>>> speakhuman.scientific(1**10, precision=1)
'1.0 x 10⁰'
>>> speakhuman.scientific(1**10, precision=0)
'1 x 10⁰'
```

## Localization

How to change locale at runtime:

```pycon
>>> import speakhuman
>>> import datetime as dt
>>> speakhuman.naturaltime(dt.timedelta(seconds=3))
'3 seconds ago'
>>> _t = speakhuman.i18n.activate("ru_RU")
>>> speakhuman.naturaltime(dt.timedelta(seconds=3))
'3 секунды назад'
>>> speakhuman.i18n.deactivate()
>>> speakhuman.naturaltime(dt.timedelta(seconds=3))
'3 seconds ago'
```

You can pass additional parameter `path` to `activate` to specify a path to search
locales in.

```pycon
>>> import speakhuman
>>> speakhuman.i18n.activate("xx_XX")
<...>
FileNotFoundError: [Errno 2] No translation file found for domain: 'speakhuman'
>>> speakhuman.i18n.activate("pt_BR", path="path/to/my/own/translation/")
<gettext.GNUTranslations instance ...>
```

<!-- usage-end -->

How to add new phrases to existing locale files:

```sh
xgettext --from-code=UTF-8 -o speakhuman.pot -k'_' -k'N_' -k'P_:1c,2' -k'NS_:1,2' -k'_ngettext:1,2' -l python src/speakhuman/*.py  # extract new phrases
msgmerge -U src/speakhuman/locale/ru_RU/LC_MESSAGES/speakhuman.po speakhuman.pot # add them to locale files
```

How to add a new locale:

```sh
msginit -i speakhuman.pot -o speakhuman/locale/<locale name>/LC_MESSAGES/speakhuman.po --locale <locale name>
```

Where `<locale name>` is a locale abbreviation, eg. `en_GB`, `pt_BR` or just `ru`, `fr`
etc.

List the language at the top of this README.
