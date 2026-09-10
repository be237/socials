# Socials frontend

Interface React + TypeScript + Vite de Socials.

## Développement

Le backend Python doit être lancé dans un terminal :

```bash
cd ../backend
source .venv/bin/activate
python -m app.main
```

Puis, depuis ce dossier :

```bash
npm install
npm run dev
```

L'API locale est disponible sur `http://localhost:3000`.

## Desktop

Tauri fournit uniquement l'enveloppe desktop et démarre le backend Python
local depuis `../backend`.

```bash
npm run tauri:dev
```
