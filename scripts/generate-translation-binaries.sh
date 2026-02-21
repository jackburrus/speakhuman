set -e

for d in src/speakhuman/locale/*/; do
    locale="$(basename $d)"
    echo "$locale"
    # compile to binary .mo
    msgfmt --check -o src/speakhuman/locale/$locale/LC_MESSAGES/speakhuman{.mo,.po}
done
