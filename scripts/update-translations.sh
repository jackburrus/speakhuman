set -e

# extract new phrases
xgettext --from-code=UTF-8 -o speakhuman.pot -k'_' -k'N_' -k'P_:1c,2' -k'NS_:1,2' -k'_ngettext:1,2' -l python src/speakhuman/*.py

for d in src/speakhuman/locale/*/; do
    locale="$(basename $d)"
    echo "$locale"
    # add them to locale files
    msgmerge -U src/speakhuman/locale/$locale/LC_MESSAGES/speakhuman.po speakhuman.pot
    # compile to binary .mo
    msgfmt --check -o src/speakhuman/locale/$locale/LC_MESSAGES/speakhuman{.mo,.po}
done
