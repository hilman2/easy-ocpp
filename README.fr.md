# easy-ocpp

🌐 [English](README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md)

[![CI](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml/badge.svg)](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hilman2/easy-ocpp)](https://github.com/hilman2/easy-ocpp/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**[Site du projet et documentation](https://hilman2.github.io/easy-ocpp/fr/)**

Serveur **OCPP auto-hébergé (CSMS)** pour bornes de recharge, conçu pour les **PME disposant de 1 à 10 bornes**. Gestion des points de charge, badges RFID, limites de recharge et rapports, sans abonnement cloud.

- **Un seul binaire, un seul fichier SQLite** – pas de base de données externe, pas de message broker.
- **OCPP 1.6J** (complet) + **OCPP 2.0.1** (ossature WebSocket, BootNotification / TransactionEvent) + **OCPP 1.5 SOAP** (ossature pour Boot/Heartbeat).
- **Interface web moderne** (Askama + htmx), comptes locaux avec mots de passe Argon2.
- Les comptes et les mots de passe se trouvent dans le fichier SQLite. Il n'y a pas de raccordement à un annuaire ni à une authentification unique.
- Fonctionne sous **Windows** (priorité) **et Linux**.

## Fonctionnalités

| Domaine         | Statut |
|-----------------|--------|
| Inventaire des wallbox + statut (en ligne/hors ligne, connecteurs, firmware) | ✅ |
| Gestion des badges RFID, apprentissage via « fenêtre d'apprentissage » (2 min, interception Authorize) | ✅ |
| Attribution badge → utilisateur ; un badge sans utilisateur est un badge invité | ✅ |
| Déverrouillage à distance pour les invités, imputation sur un label invité | ✅ |
| Valeurs en direct pendant la charge (kWh chargés, puissance actuelle, SoC) | ✅ |
| Liste des transactions, filtre par utilisateur | ✅ |
| Statistiques par mois / trimestre / année | ✅ |
| Les utilisateurs **sont** les employés : badges, recharges et limites tiennent au compte | ✅ |
| Limites par recharge : objectif en kWh et/ou minuteur, arrêt automatique | ✅ |
| Valeurs par défaut par personne, définies par l'employé ou par l'admin | ✅ |
| Libre-service employé : sa recharge en direct, ses limites, arrêt par lui-même | ✅ |
| Changement de mot de passe, exigé par un administrateur ou fait par l'utilisateur | ✅ |
| Rapport mensuel envoyé par e-mail à ceux qui ont rechargé | ✅ |
| Messages des bornes (état et pannes), conservés 60 jours, par recharge et par borne | ✅ |
| Interface multilingue (Deutsch, English, Français, Español) | ✅ |

## Démarrage

**Des binaires prêts à l'emploi** (Windows x64, Linux x64) sont disponibles sous
[Releases](https://github.com/hilman2/easy-ocpp/releases/latest). Décompresser,
lancer `easy-ocpp.exe` ou `easy-ocpp`, c'est tout (voir `INSTALL.fr.md` dans le paquet).

Ou compiler soi-même :

```bash
# une seule fois : créer la config (optionnel)
copy config.example.toml config.toml

# Build + démarrage
cargo run --release
```

Ouvrir ensuite l'interface sur <http://localhost:8080>. **Identifiants par défaut au premier démarrage :** `admin` / `admin` – merci de changer le mot de passe immédiatement sous « Utilisateurs ».

Mot de passe admin oublié ?

```bash
cargo run --release -- --reset-admin "nouveauMotDePasse123"
```

## Configurer une wallbox

Les wallbox établissent une connexion WebSocket :

```
ws://<host>:8080/ocpp/<ChargePointId>
```

Les sous-protocoles sont négociés automatiquement (`ocpp1.6` ou `ocpp2.0.1`).

Pour les appareils OCPP 1.5 plus anciens (SOAP) :

```
POST http://<host>:8080/ocpp15
```

### Mesures en direct pendant la charge

Lors de la connexion d'une wallbox OCPP 1.6, le serveur la configure
automatiquement pour qu'elle transmette toutes les 30 secondes, pendant une
charge, le relevé du compteur, la puissance et (si disponible) le SoC
(`MeterValueSampleInterval`, `MeterValuesSampledData`).
L'intervalle est réglable via `config.toml`, `0` désactive
l'auto-configuration :

```toml
[ocpp]
meter_interval_s = 30
```

Le cockpit et la page de détail de la wallbox actualisent automatiquement les
charges en cours toutes les 10 secondes (polling htmx). Si une wallbox ne
transmet pas la puissance, celle-ci est dérivée des deux derniers relevés de
compteur.

## Rôles et droits

Il n'existe qu'une seule sorte de personne : l'**utilisateur**. Un employé est un
utilisateur avec `role = user` ; ses badges, ses recharges et ses limites sont
rattachés directement à ce compte. Il n'y a plus de fiche employé distincte.

| | Admin | Employé |
|---|---|---|
| Tableau de bord, bornes, badges, gestion des utilisateurs | ✅ | – |
| Recharges de toutes les personnes | ✅ | – |
| Ses propres recharges (liste, CSV, statistiques, PDF mensuel) | ✅ | ✅ |
| Voir sa recharge en cours et l'arrêter | ✅ | ✅ |
| Objectif kWh / minuteur sur sa recharge en cours | ✅ | ✅ |
| Valeurs par défaut, les siennes | ✅ | ✅ |
| Valeurs par défaut de n'importe quel employé | ✅ | – |

Après connexion, un employé arrive sur **`/me`** et non sur le tableau de bord ;
la navigation ne lui montre que ce qu'il peut réellement ouvrir. Les restrictions
sont appliquées côté serveur, pas seulement masquées dans l'interface.

Les employés créés avant ce modèle de comptes sont repris **sans mot de passe** :
leurs recharges sont enregistrées, mais ils ne peuvent pas se connecter tant
qu'un administrateur ne leur en attribue pas un sous « Utilisateurs ».

## Limites de recharge

Une recharge s'arrête automatiquement dès qu'elle atteint une **énergie cible**
ou un **minuteur**, le premier des deux. Les deux sont facultatifs et peuvent
être désactivés séparément.

- **Valeurs par défaut par personne** : l'employé les définit sur sa propre page,
  l'administrateur sur la fiche utilisateur. Elles sont reprises au démarrage de
  chaque recharge, le minuteur comptant alors depuis le début de la charge.
- **Sur la recharge en cours** : les valeurs restent modifiables pendant la
  charge. Le minuteur y signifie « arrêter dans N minutes », ce que l'on veut
  dire quand on se tient devant la borne.

Un chien de garde vérifie toutes les 15 secondes et envoie
`RemoteStopTransaction`. La limite d'énergie est en outre vérifiée dès l'arrivée
de nouvelles mesures, afin de ne pas continuer à charger jusqu'au cycle suivant.
Si la borne refuse l'arrêt, une nouvelle tentative a lieu au cycle suivant. La
recharge n'est considérée comme traitée qu'une fois la borne d'accord.

## Recharge sur surplus et limite réseau

La page **Recharge sur surplus**, réservée à l'administrateur, active une
régulation initialement désactivée. Les badges ordinaires utilisent uniquement
le surplus. **Direction / invités** sont prioritaires. **Réseau autorisé** est
un droit distinct par badge, par exemple pour les véhicules de société; la
limite du site reste obligatoire pour tous. Les badges invités, existants et
nouveaux, sont prioritaires sans droit au réseau. À priorité égale, les voitures
partagent la puissance si leurs minimums sont disponibles; sinon certaines
attendent à 0 W et reprennent lorsque la puissance revient.

La limite par défaut est **150000 W (150 kW)** avec une marge réglable de
**10000 W**, soit une cible de 140 kW importés pour tout le bâtiment. Si les
autres consommateurs dépassent déjà cette cible, toutes les voitures sont
suspendues. Vérifier puissances minimale/maximale, phases et unité OCPP avant
activation. Les réglages s'appliquent par connecteur; les consignes en ampères
supposent 230 V par phase et sont arrondies vers le bas.

Un futur adaptateur de compteur envoie des mesures synchronisées toutes les
quelques secondes via HTTPS à `POST /api/energy/meter`, avec
`Authorization: Bearer <meter_token>`. Définir un jeton aléatoire d'au moins
24 caractères sous `[energy]` dans `config.toml` et redémarrer.
Exemple (remplacer l'horodatage par l'instant réel de mesure) :

```json
{"timestamp":"2026-10-15T10:00:00Z","grid_power_w":-5000,"charging_power_w":10000}
```

`grid_power_w` est la puissance du site : positive à l'import, négative à
l'export. `charging_power_w` est la **somme mesurée simultanément sur tous les
connecteurs gérés**, déjà incluse dans la puissance réseau. Ne pas envoyer de
compteurs d'énergie, de consignes ou de mesures OCPP prises à des instants
différents. L'exemple représente 15 kW de surplus avant les voitures.
L'adaptateur doit synchroniser les deux mesures; aucune intégration Modbus/MQTT
ou constructeur n'est encore fournie. Les horodatages périmés, répétés ou futurs
sont refusés. Les mesures en mémoire doivent être renvoyées après redémarrage.

Sans mesures récentes (15 secondes par défaut), la recharge est suspendue et
les nouveaux démarrages sont bloqués. Avec mesures récentes, une transaction
autorisée peut attendre à 0 W. Un profil par défaut à zéro protège le départ.
Les baisses sont confirmées avant toute réattribution, qui attend une nouvelle
mesure. Un refus ne libère aucun budget et déclenche une tentative d'arrêt.
Les profils positifs reviennent à zéro après au plus 30 secondes sans
renouvellement. La préautorisation locale, l'autorisation hors ligne et le
cache sont désactivés et le restent à la désactivation. Désactiver la régulation
supprime ses propres profils et permet la recharge normale. Avant la première
activation, aucun réglage des bornes n'est modifié.

**OCPP 1.6J requis**, avec Smart Charging, limites à zéro et horloge vérifiés.
Les autres protocoles sont bloqués pendant la régulation. Les profils ont été
testés sur simulateur et doivent être validés sur les bornes réelles. Une limite
stricte de 150 kW lors de pointes ou de coupures nécessite des mesures rapides
et un régulateur local; OCPP et une marge seuls ne la garantissent pas.

## Docker

```powershell
Copy-Item config.example.toml config.toml
docker compose up -d --build
```

Interface : <http://localhost:8080>. La base persiste dans le volume
`easy-ocpp-data`; `config.toml` est monté en lecture seule.
`docker compose down` conserve les données; `docker compose down -v` les supprime.

### Derrière un proxy inverse

`deploy/` contient une configuration pour un hôte où un seul Caddy dessert
plusieurs services. `compose.yaml` rattache easy-ocpp au réseau Docker du proxy
sans publier de port, et `easy-ocpp.caddy.example` est le fichier de site
correspondant.

Deux points comptent pour les bornes. La plupart parlent OCPP sans TLS : `ws://`
sur le port 80 doit donc atteindre easy-ocpp directement, car un client OCPP ne
suit pas une redirection vers HTTPS. L'exemple ne laisse passer sur le port 80
que `/ocpp/*` et `/ocpp15` et redirige tout le reste. De plus, recharger le
proxy ferme aussitôt les WebSockets ouverts, sauf si on lui accorde un délai.
C'est le rôle de `stream_close_delay`.

## Stockage des données

Tout est stocké dans **un seul fichier SQLite** sous `data/easy-ocpp.db` (modifiable via `config.toml`). Les migrations se trouvent dans `migrations/` et sont appliquées automatiquement au démarrage.

### Contrôles de cohérence à la réception des données

- **Timestamps** : >24 h dans le futur ou >10 ans dans le passé sont rejetés – repli sur l'horloge du serveur.
- **StartTransaction / StopTransaction** : idempotents face aux répétitions ; les valeurs de compteur décroissantes sont corrigées.
- **StatusNotification** : UPSERT par (wallbox, connecteur) – pas de doublons.
- **MeterValues** : les valeurs négatives sont rejetées, le SoC est validé entre 0 et 100 %, kWh → Wh normalisés.
- **Enrollment** : un tag nouvellement capturé est associé à exactement une session de fenêtre d'apprentissage ouverte.

## Structure du projet

```
src/
  main.rs           – point d'entrée, runtime Tokio, pool SQLite
  config.rs         – configuration TOML
  db.rs             – bootstrap + helpers (Argon2, paramètres)
  error.rs          – AppError / IntoResponse
  auth/             – cookies de session + login local
  domain/           – modèles de données (FromRow)
  ocpp/
    wire.rs         – parseur de trames JSON OCPP
    hub.rs          – registre de toutes les connexions actives
    ocpp16.rs       – OCPP 1.6J (complet)
    ocpp20.rs       – OCPP 2.0.1 (bootstrap)
    soap15.rs       – endpoint OCPP 1.5 SOAP
    limits.rs       – chien de garde objectif kWh / minuteur
  web/              – routeur axum, vues Askama
templates/          – templates HTML (Askama)
static/             – CSS + shim htmx (embarqué via rust-embed)
migrations/         – migrations SQLite
```

## Licence

MIT
