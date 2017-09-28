# Molten
## [WIP] Molten - Style-preserving TOML parser.

Molten is (will be) a fully lossles TOML parser with support for insretions, deletions, and editing of all items; It currently parses
and reproduces complex documents fine, modulo some bugs that are being worked on.

These days I'm adding proper tests and fixing issues that come up; Next I'll rework some of the parser internals which are currently quite a mess,
then on to error management and the actual API.

note: the master branch is not current, I'm working in the `meta` branch atm
