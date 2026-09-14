# Packaged default family

Unmodified DejaVu Sans 2.37 regular, bold, oblique and bold-oblique faces from the [official release](https://github.com/dejavu-fonts/dejavu-fonts/releases/tag/version_2_37). `LICENSE` contains the upstream redistribution terms. The exact SHA-256 identities are governed by `contracts/text-layout-v2.json`.

These resources are intended for the core's deterministic default family, embedded into the binary so the packaged application does not require a host font installation. Project font ingestion retains content-addressed copies. Do not replace these bytes as part of a routine dependency update; the layout profile and migration policy govern such changes.
