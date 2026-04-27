# Simulator Startconfig UI

Ein kleines React-Frontend (Vite + TypeScript/TSX) zum Zusammenstellen einer Startkonfiguration für den Simulator.

## Start

```sh
cd /Users/andreas/Documents/code/karpador/simulator-start-config-ui
pnpm install
pnpm dev
```

Danach im Browser öffnen (`http://localhost:5173`), Eingaben vornehmen und die JSON-Datei als Download ziehen oder in die Zwischenablage kopieren.

## Build

```sh
pnpm build
pnpm preview
```

## Verwendete Datenquellen

Aus dem bestehenden Projekt werden diese Dateien genutzt:
- `simulator/decoded_master_data/support_pokemon.json`
- `simulator/decoded_master_data/decoration.json`
- `simulator/decoded_master_data/food_base_data.json`
- `simulator/decoded_master_data/training_base_data.json`
- `simulator/decoded_master_data/competition_list.json`
- `simulator/decoded_master_data/breeder_rank.json`
- `simulator/decoded_master_data/magikarp_rank.json`

Die Dateien wurden ins Frontend unter
`public/master_data/` kopiert.
