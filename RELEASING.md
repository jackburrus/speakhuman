# Release checklist

- [ ] Get `main` to the appropriate code release state.
      [GitHub Actions](https://github.com/jackburrus/speakhuman/actions) should be
      running cleanly for all merges to `main`.
      [![GitHub Actions status](https://github.com/jackburrus/speakhuman/workflows/Test/badge.svg)](https://github.com/jackburrus/speakhuman/actions)

- [ ] Edit release draft, adjust text if needed:
      https://github.com/jackburrus/speakhuman/releases

- [ ] Check next tag is correct, amend if needed

- [ ] Publish release

- [ ] Check the tagged
      [GitHub Actions build](https://github.com/jackburrus/speakhuman/actions/workflows/release.yml)
      has released to [PyPI](https://pypi.org/project/speakhuman/#history)

- [ ] Check installation:

```bash
pip3 uninstall -y speakhuman && pip3 install -U speakhuman && python3 -c "import speakhuman; print(speakhuman.__version__)"
```
