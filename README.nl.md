# azure-cost-top

Offline CLI die een **Azure Cost Management CSV-export** leest en de duurste
posten toont per **meter**, **resourcegroep** en **tag**, met een optioneel
**verschil met de maand ervoor**.

De tests en de gemeten aantallen staan in `QUALITY.nl.md`. De Engelse versie van
deze pagina is `README.md`.

De tool leest één lokaal bestand (of standaardinvoer). Hij gebruikt geen
netwerk en geen Azure SDK, en heeft naast de standaardbibliotheek van Rust geen
dependencies.

## Installatie

Versie 0.1.0. Vanaf crates.io, zodra 0.1.0 daar staat:

```sh
cargo install azure-cost-top
```

Zelf bouwen kan met `cargo build --release`; het programma is dan
`target/release/azure-cost-top`.

## Snelstart

Vanuit een checkout van deze repository, met een Rust-toolchain:

```sh
cargo build --release
./target/release/azure-cost-top tests/fixtures/sample.csv --by meter -n 3
```

Dit leest de kleine voorbeeldexport in `tests/fixtures/` en toont de drie
duurste meters. Er wordt niets naar schijf geschreven.

## Aan de slag in 5 minuten

1. Installeer een Rust-toolchain (1.75 of nieuwer).
2. Draai `cargo build --release` in deze map.
3. Exporteer je kosten uit Azure Cost Management als CSV.
4. Draai `./target/release/azure-cost-top export.csv`.
5. Voeg `--delta` toe om de laatste maand met de maand ervoor te vergelijken.

## Gebruik

```sh
azure-cost-top export.csv                        # alle maanden, alle onderdelen
azure-cost-top export.csv --month 2026-10        # één maand
azure-cost-top export.csv --delta                # laatste maand tegen de maand ervoor
azure-cost-top export.csv -m 2026-10 --vs 2026-07
azure-cost-top export.csv --by meter,group -n 5  # twee onderdelen, 5 regels elk
azure-cost-top export.csv --by tag --tag-key team
azure-cost-top export.csv --format csv           # of json
cat export.csv | azure-cost-top -
```

`azure-cost-top --help` toont alle opties (`--by`, `--tag-key`, `--top`,
`--month`, `--delta`, `--vs`, `--currency`, `--format`).

## Voorbeeld

```sh
azure-cost-top tests/fixtures/sample.csv --delta --by group -n 3
```

```
azure-cost-top 0.1.0 - tests/fixtures/sample.csv
Period: 2026-10 vs 2026-09   Rows: 7
Total: 152.00 EUR (was 112.00, +40.00, +35.7%)

Top resource group costs (showing 3 of 3)
  #  Name      Cost  Share  Previous   Delta  Delta%
  1  rg-data  64.50  42.4%     59.50   +5.00   +8.4%
  2  rg-web   62.50  41.1%     52.50  +10.00  +19.0%
  3  rg-ops   25.00  16.4%      0.00  +25.00     new
```

De uitvoer van het programma is Engelstalig.

## Wat het doet

- **Leest de gebruikelijke exportkolommen.** De kostenkolom is
  `CostInBillingCurrency`, `PreTaxCost`, `Cost` of `CostUSD`. Andere kolommen:
  `MeterName`/`Meter`, `ResourceGroup`/`ResourceGroupName`,
  `Date`/`UsageDate`/`UsageDateTime`, `Tags`, `BillingCurrencyCode`/`Currency`.
  Hoofdletters, spaties en leestekens in de kop tellen niet mee. Datums mogen
  `MM/DD/YYYY`, `YYYY-MM-DD` (met of zonder tijd), `YYYY/MM/DD` of `YYYYMMDD`
  zijn. Een BOM en CRLF-regeleinden zijn goed.
- **Rangschikt per meter, resourcegroep en tag.** Hoogste kosten eerst, gelijke
  bedragen op naam. `-n` (`--top`) bepaalt het aantal regels per onderdeel
  (standaard 10). Namen van resourcegroepen worden hoofdletterongevoelig
  vergeleken en in kleine letters getoond. Tegoeden (negatieve kosten) worden
  meegeteld.
- **Tags.** De cel `Tags` wordt gelezen als JSON-paren, met of zonder accolades.
  Standaard één regel per paar `naam=waarde`; een rij met twee tags telt onder
  beide, dus de taglijnen kunnen samen meer zijn dan het totaal. Met
  `--tag-key team` groepeert het onderdeel op de waarde van die tag; rijen
  zonder die tag staan onder `(untagged)`. Tagnamen worden hoofdletterongevoelig
  vergeleken.
- **Verschil per maand.** `--delta` vergelijkt een maand (standaard de laatste in
  het bestand) met de maand ervoor; `--vs YYYY-MM` kiest een andere maand. Elke
  regel toont vorige kosten, verschil en verschil in procenten. Een naam die in
  de andere maand niet bestond is `new` (geen percentage).
- **Uitvoerformaten.** `table` (standaard), `csv` en `json`. CSV en JSON houden
  de volle precisie; de tabel toont twee decimalen.
- **Exacte bedragen.** Kosten worden als decimalen gelezen (negen cijfers achter
  de komma, `1.2E-5` inbegrepen) en als gehele getallen opgeteld, dus sommen
  lopen niet uit zoals bij kommagetallen.
- **Eén valuta.** Bevat de export meer valuta, dan stopt het programma en vraagt
  om `--currency CODE`. Bedragen worden nooit over valuta heen opgeteld.
- **Veilige tekst.** In de tabel en in CSV worden stuurtekens in namen (regeleinde, tab, NUL, ESC) U+FFFD, en een CSV-tekstcel die met `=`, `+`, `-` of `@` begint krijgt een enkel aanhalingsteken ervoor, zodat een spreadsheet hem niet als formule leest. Bedragen blijven getallen. Foutmeldingen op stderr worden op dezelfde manier schoongemaakt. JSON houdt de originele tekst, met escapes.
- **Alleen lezen.** Het invoerbestand wordt nooit gewijzigd of verwijderd.

## Afsluitcodes

| Code | Betekenis |
|---|---|
| 0 | Gelukt (ook bij `--help` en `--version`). |
| 1 | Probleem met de invoer. De fout noemt de regel van de export. |
| 2 | Verkeerde opdrachtregel, of een datumcel die niet gelezen kan worden. |

## Beperkingen

Alleen de CSV-export. Geen live toegang tot Azure, geen inlog, geen
valutaomrekening, geen reservering of afschrijving: het rangschikt de getallen
in het bestand.

## Onderhoud

Dit is een kleine tool met één beheerder. Een patch hoort een test te hebben en
komt binnen via Issues, zie `CONTRIBUTING.md`. Beveiligingsmeldingen gaan naar
het adres in `SECURITY.md`. Wijzigingen staan in `CHANGELOG.md`. De tool heeft
geen prijslijst en geen dependencies, dus er is niets te verversen behalve de
Rust-toolchain.

## Ontwikkeling

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo publish --dry-run
```

`QUALITY.nl.md` koppelt elke bewering hierboven aan de tests die haar dekken.

## Licentie

MIT OR Apache-2.0, zie `LICENSE-MIT` en `LICENSE-APACHE`.
