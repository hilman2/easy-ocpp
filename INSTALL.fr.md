# easy-ocpp v0.4.0 – Installation & premier démarrage (Windows)

🌐 [English](INSTALL.md) · [Deutsch](ANLEITUNG.md) · [Français](INSTALL.fr.md) · [Español](INSTALL.es.md)

Outil de gestion pour bornes de recharge (OCPP 1.5/1.6/2.0.1). Un seul binaire, un seul fichier SQLite.

## Nouveautés de la v0.4.0

- **Les employés sont désormais des utilisateurs.** Il existait jusqu'ici deux
  notions distinctes : un compte de connexion et, à côté, une fiche employé.
  Elles sont fusionnées : un employé est un utilisateur, et ses badges comme ses
  recharges sont rattachés directement à ce compte. Les employés existants sont
  repris ; ceux qui n'avaient pas de compte en obtiennent un sans mot de passe et
  ne peuvent se connecter qu'une fois qu'un administrateur leur en attribue un.
- **Les employés ne voient que leurs propres recharges.** Après connexion, ils
  arrivent sur leur propre page avec les recharges en cours de leurs badges. Le
  tableau de bord, les bornes, les badges et la gestion des utilisateurs restent
  réservés à l'administrateur.
- **Limites de recharge.** Une recharge s'arrête automatiquement dès qu'elle
  atteint un objectif en kWh ou un minuteur, selon ce qui survient en premier.
  Chaque personne dispose de valeurs par défaut appliquées à chaque nouvelle
  recharge, définies par l'employé ou par un administrateur. Sur une recharge en
  cours, les valeurs restent modifiables et l'employé peut arrêter lui-même.
- **Les badges sont sans ambiguïté.** Un badge appartient à une personne, sinon
  c'est un badge invité. La catégorie autrefois gérée séparément découle
  maintenant de l'attribution et ne peut plus la contredire.
- **Le programme s'appelle maintenant `easy-ocpp`**, détails ci-dessous.

## Nouveautés de la v0.3.0

- **Interface web multilingue :** l'interface est désormais disponible en
  Deutsch, English, Français et Español. La langue est détectée automatiquement
  à partir des réglages du navigateur et peut être changée à tout moment via le
  sélecteur dans l'en-tête.
- **Dépôt GitHub avec CI et releases automatiques :** le code source est
  désormais disponible sur <https://github.com/hilman2/easy-ocpp>. Chaque
  version est compilée automatiquement par la CI et publiée comme release.

## Nouveautés de la v0.2.0

- **Valeurs en direct pendant la charge :** le cockpit et la page de détail de
  la borne affichent, pour les charges en cours, les **kWh** déjà chargés, la
  **puissance (kW)** actuelle et, si la borne le signale, le **SoC** du
  véhicule. L'affichage se met à jour automatiquement toutes les 10 secondes.
- **Configuration automatique de la borne :** à la connexion, le serveur
  configure la borne pour qu'elle transmette des relevés toutes les 30 secondes
  (`MeterValueSampleInterval`, `MeterValuesSampledData`), réglable via la
  nouvelle section `[ocpp]` du fichier `config.toml`.
- Divers correctifs de robustesse dans le traitement des relevés (valeurs par
  phase, valeurs erronées, affichages obsolètes).

**Mise à jour depuis la v0.1.0 :** copiez simplement `easy-ocpp.exe` par-dessus
le fichier existant et redémarrez le service. La base de données
(`data\easy-ocpp.db`) reste inchangée ; il n'y a pas de nouvelles migrations.
Les valeurs en direct apparaissent à partir de la prochaine charge, une fois
que la borne s'est reconnectée.

**Renomme : `easy-occp` s'appelle desormais `easy-ocpp`.** L'ancien nom
inversait les lettres du protocole, qui s'ecrit OCPP. Lors de la mise a jour :

- Le programme s'appelle maintenant `easy-ocpp.exe`. Supprimez l'ancien
  `easy-occp.exe`, sinon deux programmes coexistent dans le dossier.
- **Votre base de donnees est conservee.** Si seul l'ancien fichier
  `data\easy-occp.db` est present, le programme continue de l'utiliser et
  l'indique au demarrage dans le journal. Pour basculer : arretez le programme,
  renommez le fichier en `easy-ocpp.db`, puis redemarrez.
- Un service installe (nssm, Planificateur de taches) doit pointer vers le
  nouveau nom de programme.
- Tout le monde est deconnecte une fois, car le cookie de session a lui aussi
  ete renomme. Il suffit de se reconnecter.

## Contenu de ce dossier

```
easy-ocpp.exe          Programme principal (Windows x64, tout inclus)
config.example.toml    Modèle de configuration
README.md              Aperçu du projet
INSTALL.fr.md          Ce guide
ANLEITUNG.md           Version allemande de ce guide
```

Le fichier `.exe` contient le schéma de base de données, l'interface web et les ressources statiques. **Rien d'autre n'a besoin d'être fourni**.

## 1. Installation

1. Copiez ce dossier à un emplacement fixe, p. ex. `C:\easy-ocpp\`.
2. Optionnel : copiez `config.example.toml` vers `config.toml` et adaptez-le (voir plus bas). Sans `config.toml`, les valeurs par défaut s'appliquent (port 8080, répertoire de données `./data`).
3. Assurez-vous que le port **8080** est libre sur l'hôte et autorisé en entrée dans le pare-feu Windows. Sinon les bornes ne peuvent pas atteindre le serveur.

## 2. Premier démarrage

Ouvrez un **PowerShell** dans le dossier et démarrez :

```powershell
.\easy-ocpp.exe
```

Au premier démarrage, l'application s'occupe de tout elle-même :

- crée le répertoire `data\`,
- crée le fichier SQLite `data\easy-ocpp.db`,
- applique le schéma complet (les migrations sont intégrées au binaire),
- crée l'utilisateur admin.

Ouvrez ensuite dans le navigateur :

<http://localhost:8080>

**Identifiants par défaut :** `admin` / `admin`. Changez le mot de passe immédiatement sous « Utilisateurs ».

Arrêtez avec `Ctrl+C` dans la fenêtre PowerShell.

## 3. Configuration (`config.toml`)

Le minimum suivant suffit :

```toml
[http]
bind = "0.0.0.0:8080"
public_base_url = "http://<ip-ou-nom-du-serveur>:8080"

[storage]
data_dir = "data"
db_file  = "easy-ocpp.db"

[ocpp]
# Intervalle en secondes auquel les bornes transmettent le relevé du compteur
# et la puissance pendant une charge. Configuré automatiquement dans la borne
# à la connexion.
# 0 = désactiver la configuration automatique.
meter_interval_s = 30
```

- `bind = "0.0.0.0:8080"`: écoute sur toutes les interfaces réseau (nécessaire pour que les bornes puissent se connecter).
- `public_base_url`: adresse publique à laquelle le serveur est joignable du point de vue des bornes.
- `meter_interval_s`: intervalle de transmission des relevés en direct (30 s par défaut).

Les sections LDAP / Entra ID sont commentées dans l'exemple, ce sont pour l'instant des ébauches, non actives.

## 4. Configurer une borne

Saisissez l'URL du backend dans la configuration de l'appareil (portail du fabricant ou interface web de la borne) :

**OCPP 1.6 / 2.0.1 (WebSocket) :**
```
ws://<server-ip>:8080/ocpp/<ChargePointId>
```

`<ChargePointId>` est l'identifiant unique de la borne (librement choisi, p. ex. `WB-Halle-01`). Le sous-protocole (`ocpp1.6` / `ocpp2.0.1`) est négocié automatiquement.

**OCPP 1.5 (SOAP, appareils anciens uniquement) :**
```
http://<server-ip>:8080/ocpp15
```

Dès que la borne se connecte, elle apparaît sous **Bornes** avec le statut *online*. Peu après la connexion, le serveur configure automatiquement la borne pour les relevés en direct (OCPP 1.6 uniquement).

## 5. Enregistrer des badges RFID

1. Dans l'interface → **Badges → Ouvrir la fenêtre d'apprentissage** (active pendant 2 minutes).
2. Authentifiez-vous à la borne avec le nouveau badge RFID.
3. Le badge apparaît dans la liste et peut être attribué à un employé ou marqué comme badge invité (avec date d'expiration).

## 6. Exécution permanente comme service Windows (optionnel)

`easy-ocpp.exe` est un programme console ordinaire. Pour un démarrage automatique/service :

**Variante A : Planificateur de tâches**

- Planificateur de tâches → *Créer une tâche* → Déclencheur *Au démarrage du système* → Action *Démarrer un programme* → `C:\easy-ocpp\easy-ocpp.exe` → *Démarrer dans :* `C:\easy-ocpp\`.
- Activez *« Exécuter même si l'utilisateur n'est pas connecté »*.

**Variante B : NSSM (Non-Sucking Service Manager)**

```powershell
nssm install easy-ocpp "C:\easy-ocpp\easy-ocpp.exe"
nssm set easy-ocpp AppDirectory "C:\easy-ocpp"
nssm start easy-ocpp
```

## 7. Réinitialiser le mot de passe admin

Si le mot de passe admin est perdu :

```powershell
.\easy-ocpp.exe --reset-admin "nouveauMotDePasse123"
```

Réinitialise le mot de passe du compte `admin` puis se termine.

## 8. Sauvegarde

Tout l'essentiel se trouve dans **un seul fichier** : `data\easy-ocpp.db`.

Pour une sauvegarde cohérente, arrêtez brièvement le service et copiez le fichier (y compris les éventuels `-wal` / `-shm`). C'est tout.

## 9. Dépannage

| Problème | Cause / solution |
|---------|------------------|
| Le navigateur affiche « page inaccessible » | Port 8080 occupé ou bloqué par le pare-feu. Vérifiez avec `netstat -ano \| findstr :8080`. |
| La borne ne se connecte pas | Vérifiez `public_base_url` / le pare-feu / l'URL avec le `ChargePointId`. Les logs de la console montrent les connexions WS entrantes. |
| Pas de valeurs en direct pendant la charge | Laissez la borne se reconnecter (la configuration est appliquée à la connexion). Vérifiez le log console : `MeterValueSampleInterval` doit apparaître comme « défini ». Certaines bornes nécessitent un redémarrage, d'autres ne prennent pas en charge la mesure de puissance, auquel cas la puissance est dérivée des relevés du compteur. |
| « database is locked » | Ne lancez pas un second `easy-ocpp.exe` en même temps sur la même base. |
| Plus de logs souhaités | Avant le démarrage : définissez `$env:RUST_LOG="debug"`. |

## 10. Désinstallation

Supprimez le service/la tâche, supprimez le dossier. Il n'y a ni entrées de registre ni dépendances externes.

---

Version 0.4.0 · Licence : MIT
