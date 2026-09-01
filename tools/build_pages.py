# -*- coding: utf-8 -*-
"""Erzeugt die statische GitHub-Pages-Seite in docs/. Vier Sprachen,
je eine eigene URL mit hreflang, Canonical, Open Graph und JSON-LD."""
import io
import json
import os

BASE = 'https://hilman2.github.io/easy-ocpp/'
REPO = 'https://github.com/hilman2/easy-ocpp'
OG_IMAGE = 'https://opengraph.githubassets.com/1/hilman2/easy-ocpp'

LANGS = ['en', 'de', 'fr', 'es']
LANG_NAMES = {'en': 'English', 'de': 'Deutsch', 'fr': 'Français', 'es': 'Español'}
# en liegt in der Wurzel, die anderen in Unterordnern.
PATHS = {'en': '', 'de': 'de/', 'fr': 'fr/', 'es': 'es/'}

C = {}

# ============================================================== English =====
C['en'] = {
    'title': 'Self-hosted OCPP CSMS for small businesses | easy-ocpp',
    'desc': 'Open-source OCPP 1.6J / 2.0.1 CSMS for 1–10 wallboxes: RFID chips, '
            'charging limits, live meter values, reports. One Rust binary, one SQLite file.',
    'h1': 'Your own OCPP server for up to ten wallboxes',
    'lede': 'easy-ocpp is a self-hosted charging station management system (CSMS) '
            'for small and medium businesses. It speaks OCPP 1.6J, 2.0.1 and 1.5 SOAP. '
            'One binary, one SQLite file, no cloud subscription and no permanent '
            'internet connection.',
    'cta_download': 'Download release',
    'cta_source': 'Source on GitHub',
    'nav_features': 'Features',
    'nav_roles': 'Roles',
    'nav_limits': 'Charging limits',
    'nav_start': 'Getting started',
    'nav_faq': 'FAQ',
    'pitch': [
        ('No cloud, no subscription',
         'Everything runs on a machine you control. A small server is enough, or a NUC, '
         'or a spare office PC. Charging data never leaves your network.'),
        ('One binary, one file',
         'No database server, no message broker, no container stack. Unpack, start, done. '
         'Your entire dataset is one SQLite file you can copy for a backup.'),
        ('Built for 1–10 wallboxes',
         'Deliberately not an enterprise platform. The features a company with a handful '
         'of charge points actually needs, without the operational overhead.'),
    ],
    'features_h': 'What it does',
    'features': [
        ('Wallbox inventory', 'Online status, connectors, firmware and serial per charge point, live from the OCPP connection.'),
        ('RFID chip management', 'New chips are enrolled through a two-minute learning window: hold the chip to a wallbox, the server captures the tag.'),
        ('Chips and people', 'A chip either belongs to a person or it is a guest chip. The category follows the assignment, so the two cannot contradict each other.'),
        ('Live meter values', 'Charged kWh, current power and state of charge while a session runs, refreshed every ten seconds.'),
        ('Charging limits', 'Stop automatically at a target energy or after a set time, whichever comes first.'),
        ('Guest charging', 'Unlock a session remotely for a visitor and book it against a guest label.'),
        ('Reports', 'Session list with CSV export, statistics by month, quarter and year, and a monthly PDF per person.'),
        ('Four languages', 'The whole interface in German, English, French and Spanish, switchable per user.'),
    ],
    'roles_h': 'Two roles, clearly separated',
    'roles_lede': 'There is exactly one kind of person in the system: a user. An employee '
                  'is a user, and their chips, charging sessions and limits hang off that '
                  'account directly. What an employee may do is deliberately narrow.',
    'roles_head': ['', 'Admin', 'Employee'],
    'roles': [
        ('Wallboxes, chips, user management', True, False),
        ('Charging sessions of everyone', True, False),
        ('Own charging sessions, CSV, statistics, monthly PDF', True, True),
        ('Watch own running session and stop it', True, True),
        ('Target kWh and timer on own session', True, True),
        ('Default limits for any employee', True, False),
    ],
    'roles_note': 'The server enforces these restrictions. Hiding things in the '
                  'interface would not be enough. After signing in, an employee lands '
                  'on their own page, and the navigation shows only what they can '
                  'actually open.',
    'limits_h': 'Charging limits that actually stop the session',
    'limits_body': 'A session ends by itself once it reaches a target energy or a timer, '
                   'whichever comes first. Both are optional. Every person has defaults '
                   'that apply to each new session, set either by the employee or by an '
                   'administrator; on a running session the values can still be changed.',
    'limits_tech': 'A watchdog checks every 15 seconds and sends RemoteStopTransaction. '
                   'The energy limit is additionally checked the moment new meter values '
                   'arrive, so charging does not run past the target until the next tick. '
                   'If a wallbox rejects the stop, it is retried.',
    'start_h': 'Getting started',
    'start_body': 'Grab a release for Windows or Linux, unpack it, run it. The interface '
                  'is then at localhost port 8080; the first login is admin / admin.',
    'start_build': 'Or build from source:',
    'connect_h': 'Connecting a wallbox',
    'connect_body': 'Point the charge point at your server. Subprotocols are negotiated '
                    'automatically, and an unknown charge point registers itself on first '
                    'connection.',
    'connect_soap': 'For older OCPP 1.5 devices over SOAP:',
    'compat_h': 'Protocol support',
    'compat_head': ['Version', 'Transport', 'Status'],
    'compat': [
        ('OCPP 1.6J', 'WebSocket', 'Complete (boot, heartbeat, status, authorize, transactions, meter values, remote start/stop, configuration)'),
        ('OCPP 2.0.1', 'WebSocket', 'Scaffold (BootNotification and TransactionEvent)'),
        ('OCPP 1.5', 'SOAP', 'Scaffold (boot and heartbeat)'),
    ],
    'faq_h': 'Frequently asked questions',
    'faq': [
        ('What is a CSMS?',
         'A Charging Station Management System is the backend an EV charge point talks to. '
         'It authorises RFID cards, records charging sessions and meter readings, and can '
         'start or stop a session remotely. OCPP is the protocol they use to talk.'),
        ('Will it work with my wallbox?',
         'Any charge point that speaks OCPP 1.6J over WebSocket should work, which covers '
         'the large majority of devices sold in the last several years. Older hardware '
         'limited to OCPP 1.5 SOAP can connect for boot and heartbeat.'),
        ('Do I need an internet connection?',
         'No. The wallboxes connect to your server over the local network. An internet '
         'connection is only needed if you want to reach the interface from outside.'),
        ('How do I see who charged how much?',
         'Every session is tied to the RFID chip that started it, and every chip belongs '
         'to a person. The session list, the statistics and the monthly PDF are grouped '
         'accordingly. Sessions on unassigned chips are booked as guest charging.'),
        ('Can employees see each other\'s charging?',
         'No. An employee only ever sees the sessions started with their own chips. Only '
         'an administrator sees everything.'),
        ('What does it cost?',
         'Nothing. easy-ocpp is open source under the MIT licence. There is no paid tier '
         'and no per-charge-point fee.'),
    ],
    'footer_license': 'MIT licensed. Not affiliated with the Open Charge Alliance.',
    'footer_lang': 'Language',
}

# =============================================================== Deutsch =====
C['de'] = {
    'title': 'Selbst gehostetes OCPP-CSMS für KMU | easy-ocpp',
    'desc': 'Open-Source-CSMS für OCPP 1.6J / 2.0.1 und 1–10 Wallboxen: RFID-Chips, '
            'Ladelimits, Live-Messwerte, Auswertungen. Ein Rust-Binary, eine SQLite-Datei.',
    'h1': 'Ihr eigener OCPP-Server für bis zu zehn Wallboxen',
    'lede': 'easy-ocpp ist ein selbst gehostetes Ladepunkt-Management-System (CSMS) für '
            'kleine und mittlere Unternehmen. Es spricht OCPP 1.6J, 2.0.1 und 1.5 SOAP. '
            'Ein Programm, eine SQLite-Datei, kein Cloud-Abo und keine dauerhafte '
            'Internetverbindung.',
    'cta_download': 'Release herunterladen',
    'cta_source': 'Quellcode auf GitHub',
    'nav_features': 'Funktionen',
    'nav_roles': 'Rollen',
    'nav_limits': 'Ladelimits',
    'nav_start': 'Loslegen',
    'nav_faq': 'Fragen',
    'pitch': [
        ('Keine Cloud, kein Abo',
         'Alles läuft auf einem Rechner, der Ihnen gehört. Ein kleiner Server reicht, ein '
         'NUC, notfalls ein übriger Büro-PC. Ladedaten verlassen Ihr Netz nicht.'),
        ('Ein Programm, eine Datei',
         'Kein Datenbankserver, kein Message-Broker, kein Container-Stack. Entpacken, '
         'starten, fertig. Der gesamte Datenbestand ist eine SQLite-Datei, die Sie zum '
         'Sichern einfach kopieren.'),
        ('Gebaut für 1–10 Wallboxen',
         'Bewusst keine Konzern-Plattform. Das, was ein Betrieb mit einer Handvoll '
         'Ladepunkte wirklich braucht, ohne den Betriebsaufwand drumherum.'),
    ],
    'features_h': 'Was es kann',
    'features': [
        ('Wallbox-Inventar', 'Online-Status, Connectors, Firmware und Seriennummer je Ladepunkt, direkt aus der OCPP-Verbindung.'),
        ('Chip-Verwaltung', 'Neue Chips werden über ein Lernfenster von zwei Minuten angelernt: Chip an die Wallbox halten, der Server fängt den Tag ab.'),
        ('Chips und Personen', 'Ein Chip gehört entweder zu einer Person oder er ist ein Gast-Chip. Die Kategorie folgt der Zuordnung und kann ihr nicht widersprechen.'),
        ('Live-Messwerte', 'Geladene kWh, aktuelle Leistung und Ladestand während der Ladung, alle zehn Sekunden aktualisiert.'),
        ('Ladelimits', 'Automatische Abschaltung bei erreichter Ziel-Energie oder abgelaufener Zeit, je nachdem was zuerst eintritt.'),
        ('Gast-Ladungen', 'Für Besucher aus der Ferne freischalten und auf ein Gast-Label buchen.'),
        ('Auswertungen', 'Ladungsliste mit CSV-Export, Statistik nach Monat, Quartal und Jahr sowie ein Monats-PDF je Person.'),
        ('Vier Sprachen', 'Die gesamte Oberfläche auf Deutsch, Englisch, Französisch und Spanisch, pro Benutzer umschaltbar.'),
    ],
    'roles_h': 'Zwei Rollen, sauber getrennt',
    'roles_lede': 'Es gibt genau eine Sorte Person im System: den Benutzer. Ein Mitarbeiter '
                  'ist ein Benutzer, und seine Chips, Ladungen und Limits hängen direkt an '
                  'diesem Konto. Was ein Mitarbeiter darf, ist bewusst eng gefasst.',
    'roles_head': ['', 'Admin', 'Mitarbeiter'],
    'roles': [
        ('Wallboxen, Chips, Benutzerverwaltung', True, False),
        ('Ladungen aller Personen', True, False),
        ('Eigene Ladungen, CSV, Statistik, Monats-PDF', True, True),
        ('Eigene laufende Ladung sehen und beenden', True, True),
        ('Ziel-kWh und Timer an der eigenen Ladung', True, True),
        ('Standardvorgaben für jeden Mitarbeiter', True, False),
    ],
    'roles_note': 'Die Einschränkungen greifen auf dem Server. Etwas in der Oberfläche '
                  'auszublenden würde nicht genügen. Nach dem Anmelden landet ein '
                  'Mitarbeiter auf seiner eigenen Seite, und die Navigation zeigt ihm '
                  'nur, was er auch öffnen darf.',
    'limits_h': 'Ladelimits, die tatsächlich abschalten',
    'limits_body': 'Eine Ladung endet von selbst, sobald sie eine Ziel-Energie oder einen '
                   'Timer erreicht, je nachdem was zuerst eintritt. Beides ist optional. Jede Person '
                   'hat Standardvorgaben, die für jede neue Ladung gelten und die entweder '
                   'der Mitarbeiter selbst oder ein Administrator setzt; an der laufenden '
                   'Ladung lassen sich die Werte weiterhin ändern.',
    'limits_tech': 'Ein Watchdog prüft alle 15 Sekunden und schickt RemoteStopTransaction. '
                   'Das Energielimit wird zusätzlich sofort beim Eintreffen neuer Messwerte '
                   'geprüft, damit nicht bis zum nächsten Takt über das Ziel hinaus geladen '
                   'wird. Lehnt eine Wallbox den Stop ab, wird es erneut versucht.',
    'start_h': 'Loslegen',
    'start_body': 'Ein Release für Windows oder Linux holen, entpacken, starten. Die '
                  'Oberfläche liegt dann auf Port 8080; der erste Login ist admin / admin.',
    'start_build': 'Oder selbst aus dem Quellcode bauen:',
    'connect_h': 'Wallbox verbinden',
    'connect_body': 'Den Ladepunkt auf Ihren Server zeigen lassen. Die Subprotokolle werden '
                    'automatisch ausgehandelt, und ein unbekannter Ladepunkt trägt sich beim '
                    'ersten Verbinden selbst ein.',
    'connect_soap': 'Für ältere OCPP-1.5-Geräte über SOAP:',
    'compat_h': 'Protokoll-Unterstützung',
    'compat_head': ['Version', 'Transport', 'Stand'],
    'compat': [
        ('OCPP 1.6J', 'WebSocket', 'Vollständig (Boot, Heartbeat, Status, Authorize, Transaktionen, Messwerte, Remote-Start/Stop, Konfiguration)'),
        ('OCPP 2.0.1', 'WebSocket', 'Gerüst (BootNotification und TransactionEvent)'),
        ('OCPP 1.5', 'SOAP', 'Gerüst (Boot und Heartbeat)'),
    ],
    'faq_h': 'Häufige Fragen',
    'faq': [
        ('Was ist ein CSMS?',
         'Ein Charging Station Management System ist das Backend, mit dem eine Ladestation '
         'spricht. Es autorisiert RFID-Karten, zeichnet Ladevorgänge und Zählerstände auf '
         'und kann Ladungen aus der Ferne starten oder beenden. OCPP ist das Protokoll, '
         'über das die beiden sich unterhalten.'),
        ('Funktioniert das mit meiner Wallbox?',
         'Jeder Ladepunkt, der OCPP 1.6J über WebSocket spricht, sollte laufen. Das trifft '
         'auf die große Mehrheit der in den letzten Jahren verkauften Geräte zu. Ältere '
         'Hardware, die nur OCPP 1.5 SOAP kann, verbindet sich für Boot und Heartbeat.'),
        ('Brauche ich eine Internetverbindung?',
         'Nein. Die Wallboxen verbinden sich über das lokale Netz mit Ihrem Server. Internet '
         'brauchen Sie nur, wenn Sie die Oberfläche von außen erreichen wollen.'),
        ('Wie sehe ich, wer wie viel geladen hat?',
         'Jede Ladung hängt an dem RFID-Chip, mit dem sie gestartet wurde, und jeder Chip '
         'gehört zu einer Person. Ladungsliste, Statistik und Monats-PDF sind entsprechend '
         'gruppiert. Ladungen auf nicht zugeordneten Chips laufen als Gast-Ladung.'),
        ('Sehen Mitarbeiter die Ladungen der anderen?',
         'Nein. Ein Mitarbeiter sieht ausschließlich die Ladungen, die mit seinen eigenen '
         'Chips gestartet wurden. Nur ein Administrator sieht alles.'),
        ('Was kostet es?',
         'Nichts. easy-ocpp ist Open Source unter der MIT-Lizenz. Es gibt keine kostenpflichtige '
         'Variante und keine Gebühr je Ladepunkt.'),
    ],
    'footer_license': 'MIT-Lizenz. Nicht mit der Open Charge Alliance verbunden.',
    'footer_lang': 'Sprache',
}

# ============================================================== Français =====
C['fr'] = {
    'title': 'CSMS OCPP auto-hébergé pour PME | easy-ocpp',
    'desc': 'CSMS open source pour OCPP 1.6J / 2.0.1 et 1 à 10 bornes : badges RFID, '
            'limites de recharge, mesures en direct, rapports. Un binaire Rust, un fichier SQLite.',
    'h1': 'Votre propre serveur OCPP pour dix bornes au plus',
    'lede': 'easy-ocpp est un système de gestion de points de charge (CSMS) auto-hébergé '
            'pour les petites et moyennes entreprises. Il parle OCPP 1.6J, 2.0.1 et 1.5 SOAP. '
            'Un seul binaire, un seul fichier SQLite, sans abonnement cloud ni connexion '
            'internet permanente.',
    'cta_download': 'Télécharger une version',
    'cta_source': 'Code source sur GitHub',
    'nav_features': 'Fonctionnalités',
    'nav_roles': 'Rôles',
    'nav_limits': 'Limites',
    'nav_start': 'Démarrer',
    'nav_faq': 'Questions',
    'pitch': [
        ('Pas de cloud, pas d\'abonnement',
         'Tout tourne sur une machine qui vous appartient. Un petit serveur suffit, un NUC, '
         'au besoin un PC de bureau inutilisé. Les données de recharge ne quittent pas '
         'votre réseau.'),
        ('Un binaire, un fichier',
         'Pas de serveur de base de données, pas de courtier de messages, pas de pile de '
         'conteneurs. Décompressez, lancez, c\'est fait. Toutes vos données tiennent dans un '
         'fichier SQLite qu\'il suffit de copier pour la sauvegarde.'),
        ('Conçu pour 1 à 10 bornes',
         'Volontairement pas une plateforme d\'entreprise. Ce dont une société avec quelques '
         'points de charge a réellement besoin, sans la charge d\'exploitation qui va avec.'),
    ],
    'features_h': 'Ce qu\'il fait',
    'features': [
        ('Inventaire des bornes', 'État en ligne, connecteurs, micrologiciel et numéro de série par point de charge, directement depuis la liaison OCPP.'),
        ('Gestion des badges', 'Les nouveaux badges sont enregistrés via une fenêtre d\'apprentissage de deux minutes : présentez le badge, le serveur intercepte le tag.'),
        ('Badges et personnes', 'Un badge appartient à une personne, sinon c\'est un badge invité. La catégorie découle de l\'attribution et ne peut pas la contredire.'),
        ('Mesures en direct', 'kWh chargés, puissance actuelle et état de charge pendant la recharge, actualisés toutes les dix secondes.'),
        ('Limites de recharge', 'Arrêt automatique à l\'énergie cible ou au bout d\'une durée définie, selon ce qui arrive en premier.'),
        ('Recharges invités', 'Déverrouillez à distance pour un visiteur et imputez la recharge à un label invité.'),
        ('Rapports', 'Liste des recharges avec export CSV, statistiques par mois, trimestre et année, et un PDF mensuel par personne.'),
        ('Quatre langues', 'Toute l\'interface en allemand, anglais, français et espagnol, au choix de chaque utilisateur.'),
    ],
    'roles_h': 'Deux rôles, nettement séparés',
    'roles_lede': 'Il n\'existe qu\'une seule sorte de personne dans le système : '
                  'l\'utilisateur. Un employé est un utilisateur, et ses badges, ses '
                  'recharges et ses limites sont rattachés directement à ce compte. Ce qu\'un '
                  'employé peut faire est volontairement restreint.',
    'roles_head': ['', 'Admin', 'Employé'],
    'roles': [
        ('Bornes, badges, gestion des utilisateurs', True, False),
        ('Recharges de toutes les personnes', True, False),
        ('Ses propres recharges, CSV, statistiques, PDF mensuel', True, True),
        ('Voir sa recharge en cours et l\'arrêter', True, True),
        ('Objectif kWh et minuteur sur sa recharge', True, True),
        ('Valeurs par défaut de n\'importe quel employé', True, False),
    ],
    'roles_note': 'Le serveur applique ces restrictions. Les masquer dans l\'interface '
                  'ne suffirait pas. Après connexion, un employé arrive sur sa propre '
                  'page et la navigation ne lui montre que ce qu\'il peut réellement '
                  'ouvrir.',
    'limits_h': 'Des limites qui arrêtent vraiment la recharge',
    'limits_body': 'Une recharge se termine d\'elle-même dès qu\'elle atteint une énergie '
                   'cible ou un minuteur, selon ce qui survient en premier. Les deux sont facultatifs. '
                   'Chaque personne dispose de valeurs par défaut appliquées à chaque nouvelle '
                   'recharge, définies par l\'employé lui-même ou par un administrateur ; sur '
                   'une recharge en cours, les valeurs restent modifiables.',
    'limits_tech': 'Un chien de garde vérifie toutes les 15 secondes et envoie '
                   'RemoteStopTransaction. La limite d\'énergie est en outre contrôlée dès '
                   'l\'arrivée de nouvelles mesures, afin de ne pas dépasser l\'objectif en '
                   'attendant le cycle suivant. Si une borne refuse l\'arrêt, une nouvelle '
                   'tentative est faite.',
    'start_h': 'Démarrer',
    'start_body': 'Récupérez une version pour Windows ou Linux, décompressez, lancez. '
                  'L\'interface se trouve alors sur le port 8080 ; la première connexion est '
                  'admin / admin.',
    'start_build': 'Ou compilez depuis les sources :',
    'connect_h': 'Connecter une borne',
    'connect_body': 'Faites pointer le point de charge vers votre serveur. Les sous-protocoles '
                    'sont négociés automatiquement, et un point de charge inconnu s\'enregistre '
                    'de lui-même à la première connexion.',
    'connect_soap': 'Pour les anciens appareils OCPP 1.5 en SOAP :',
    'compat_h': 'Protocoles pris en charge',
    'compat_head': ['Version', 'Transport', 'État'],
    'compat': [
        ('OCPP 1.6J', 'WebSocket', 'Complet (boot, heartbeat, statut, autorisation, transactions, mesures, démarrage/arrêt à distance, configuration)'),
        ('OCPP 2.0.1', 'WebSocket', 'Ébauche (BootNotification et TransactionEvent)'),
        ('OCPP 1.5', 'SOAP', 'Ébauche (boot et heartbeat)'),
    ],
    'faq_h': 'Questions fréquentes',
    'faq': [
        ('Qu\'est-ce qu\'un CSMS ?',
         'Un Charging Station Management System est le backend avec lequel une borne de '
         'recharge dialogue. Il autorise les cartes RFID, enregistre les recharges et les '
         'index de compteur, et peut démarrer ou arrêter une recharge à distance. OCPP est '
         'le protocole qu\'ils utilisent pour se parler.'),
        ('Est-ce compatible avec ma borne ?',
         'Tout point de charge parlant OCPP 1.6J en WebSocket devrait fonctionner, ce qui '
         'couvre la grande majorité des appareils vendus ces dernières années. Le matériel '
         'plus ancien limité à OCPP 1.5 SOAP peut se connecter pour le boot et le heartbeat.'),
        ('Ai-je besoin d\'une connexion internet ?',
         'Non. Les bornes se connectent à votre serveur par le réseau local. Internet n\'est '
         'nécessaire que si vous souhaitez atteindre l\'interface depuis l\'extérieur.'),
        ('Comment voir qui a rechargé combien ?',
         'Chaque recharge est rattachée au badge RFID qui l\'a démarrée, et chaque badge '
         'appartient à une personne. La liste, les statistiques et le PDF mensuel sont '
         'regroupés en conséquence. Les recharges sur badge non attribué comptent comme '
         'recharges invités.'),
        ('Les employés voient-ils les recharges des autres ?',
         'Non. Un employé ne voit que les recharges démarrées avec ses propres badges. Seul '
         'un administrateur voit tout.'),
        ('Combien ça coûte ?',
         'Rien. easy-ocpp est open source sous licence MIT. Il n\'y a ni version payante ni '
         'redevance par point de charge.'),
    ],
    'footer_license': 'Sous licence MIT. Sans lien avec l\'Open Charge Alliance.',
    'footer_lang': 'Langue',
}

# =============================================================== Español =====
C['es'] = {
    'title': 'CSMS OCPP autoalojado para pymes | easy-ocpp',
    'desc': 'CSMS de código abierto para OCPP 1.6J / 2.0.1 y 1–10 cargadores: tarjetas RFID, '
            'límites de carga, mediciones en vivo, informes. Un binario Rust, un archivo SQLite.',
    'h1': 'Su propio servidor OCPP para hasta diez cargadores',
    'lede': 'easy-ocpp es un sistema de gestión de puntos de carga (CSMS) autoalojado para '
            'pequeñas y medianas empresas. Habla OCPP 1.6J, 2.0.1 y 1.5 SOAP. Un solo '
            'binario, un solo archivo SQLite, sin suscripción en la nube ni conexión '
            'permanente a internet.',
    'cta_download': 'Descargar versión',
    'cta_source': 'Código fuente en GitHub',
    'nav_features': 'Funciones',
    'nav_roles': 'Roles',
    'nav_limits': 'Límites',
    'nav_start': 'Empezar',
    'nav_faq': 'Preguntas',
    'pitch': [
        ('Sin nube, sin suscripción',
         'Todo funciona en una máquina suya. Basta un servidor pequeño, un NUC o incluso un '
         'PC de oficina que le sobre. Los datos de carga no salen de su red.'),
        ('Un binario, un archivo',
         'Sin servidor de base de datos, sin broker de mensajes, sin pila de contenedores. '
         'Descomprima, ejecute y listo. Todos sus datos caben en un archivo SQLite que basta '
         'copiar para hacer una copia de seguridad.'),
        ('Pensado para 1–10 cargadores',
         'Deliberadamente no es una plataforma corporativa. Lo que una empresa con unos pocos '
         'puntos de carga necesita de verdad, sin la carga operativa que suele acompañarla.'),
    ],
    'features_h': 'Qué hace',
    'features': [
        ('Inventario de cargadores', 'Estado en línea, conectores, firmware y número de serie por punto de carga, directamente desde la conexión OCPP.'),
        ('Gestión de tarjetas', 'Las tarjetas nuevas se dan de alta mediante una ventana de aprendizaje de dos minutos: acerque la tarjeta y el servidor captura el tag.'),
        ('Tarjetas y personas', 'Una tarjeta pertenece a una persona o es de invitado. La categoría se deriva de la asignación y no puede contradecirla.'),
        ('Mediciones en vivo', 'kWh cargados, potencia actual y estado de carga durante la sesión, actualizados cada diez segundos.'),
        ('Límites de carga', 'Parada automática al alcanzar la energía objetivo o cumplirse el tiempo, lo que ocurra primero.'),
        ('Cargas de invitados', 'Desbloquee de forma remota para una visita e impute la carga a una etiqueta de invitado.'),
        ('Informes', 'Lista de cargas con exportación CSV, estadísticas por mes, trimestre y año, y un PDF mensual por persona.'),
        ('Cuatro idiomas', 'Toda la interfaz en alemán, inglés, francés y español, conmutable por usuario.'),
    ],
    'roles_h': 'Dos roles, claramente separados',
    'roles_lede': 'Solo existe un tipo de persona en el sistema: el usuario. Un empleado es '
                  'un usuario, y sus tarjetas, sus cargas y sus límites cuelgan directamente '
                  'de esa cuenta. Lo que puede hacer un empleado está deliberadamente acotado.',
    'roles_head': ['', 'Admin', 'Empleado'],
    'roles': [
        ('Cargadores, tarjetas, gestión de usuarios', True, False),
        ('Cargas de todas las personas', True, False),
        ('Sus propias cargas, CSV, estadísticas, PDF mensual', True, True),
        ('Ver su carga en curso y detenerla', True, True),
        ('kWh objetivo y temporizador en su carga', True, True),
        ('Valores predeterminados de cualquier empleado', True, False),
    ],
    'roles_note': 'El servidor aplica estas restricciones. Ocultarlas en la interfaz no '
                  'bastaría. Tras iniciar sesión, un empleado llega a su propia página '
                  'y la navegación solo le muestra lo que realmente puede abrir.',
    'limits_h': 'Límites que detienen la carga de verdad',
    'limits_body': 'Una carga termina sola en cuanto alcanza una energía objetivo o un '
                   'temporizador, lo que ocurra primero. Ambos son opcionales. Cada persona '
                   'tiene valores predeterminados que se aplican a cada carga nueva y que '
                   'fija el propio empleado o un administrador; en una carga en curso los '
                   'valores siguen siendo modificables.',
    'limits_tech': 'Un vigilante comprueba cada 15 segundos y envía RemoteStopTransaction. '
                   'El límite de energía se comprueba además en cuanto llegan nuevas '
                   'mediciones, para no sobrepasar el objetivo esperando al siguiente ciclo. '
                   'Si un cargador rechaza la parada, se vuelve a intentar.',
    'start_h': 'Empezar',
    'start_body': 'Descargue una versión para Windows o Linux, descomprima y ejecute. La '
                  'interfaz queda entonces en el puerto 8080; el primer acceso es admin / admin.',
    'start_build': 'O compile desde el código fuente:',
    'connect_h': 'Conectar un cargador',
    'connect_body': 'Apunte el punto de carga a su servidor. Los subprotocolos se negocian '
                    'automáticamente, y un punto de carga desconocido se registra solo en la '
                    'primera conexión.',
    'connect_soap': 'Para equipos OCPP 1.5 antiguos por SOAP:',
    'compat_h': 'Protocolos admitidos',
    'compat_head': ['Versión', 'Transporte', 'Estado'],
    'compat': [
        ('OCPP 1.6J', 'WebSocket', 'Completo (boot, heartbeat, estado, autorización, transacciones, mediciones, arranque/parada remotos, configuración)'),
        ('OCPP 2.0.1', 'WebSocket', 'Esqueleto (BootNotification y TransactionEvent)'),
        ('OCPP 1.5', 'SOAP', 'Esqueleto (boot y heartbeat)'),
    ],
    'faq_h': 'Preguntas frecuentes',
    'faq': [
        ('¿Qué es un CSMS?',
         'Un Charging Station Management System es el backend con el que habla un punto de '
         'recarga. Autoriza tarjetas RFID, registra las cargas y las lecturas del contador y '
         'puede iniciar o detener una carga de forma remota. OCPP es el protocolo con el que '
         'ambos se comunican.'),
        ('¿Funciona con mi cargador?',
         'Cualquier punto de carga que hable OCPP 1.6J sobre WebSocket debería funcionar, lo '
         'que cubre la gran mayoría de equipos vendidos en los últimos años. El hardware más '
         'antiguo limitado a OCPP 1.5 SOAP puede conectarse para boot y heartbeat.'),
        ('¿Necesito conexión a internet?',
         'No. Los cargadores se conectan a su servidor por la red local. Internet solo hace '
         'falta si quiere acceder a la interfaz desde fuera.'),
        ('¿Cómo veo quién ha cargado cuánto?',
         'Cada carga va ligada a la tarjeta RFID con la que se inició, y cada tarjeta '
         'pertenece a una persona. La lista, las estadísticas y el PDF mensual se agrupan en '
         'consecuencia. Las cargas con tarjetas sin asignar se contabilizan como cargas de '
         'invitado.'),
        ('¿Ven los empleados las cargas de los demás?',
         'No. Un empleado solo ve las cargas iniciadas con sus propias tarjetas. Únicamente '
         'un administrador lo ve todo.'),
        ('¿Cuánto cuesta?',
         'Nada. easy-ocpp es código abierto bajo licencia MIT. No hay versión de pago ni '
         'tarifa por punto de carga.'),
    ],
    'footer_license': 'Licencia MIT. Sin relación con la Open Charge Alliance.',
    'footer_lang': 'Idioma',
}



# ---------------------------------------------------------------------------
# Rechtsseiten. Anbieterangaben nach § 5 DDG und Hinweise nach Art. 13 DSGVO.
# Massgeblich ist die deutsche Fassung, die uebrigen sind Uebersetzungen.
# ---------------------------------------------------------------------------

ANBIETER = {
    'name': 'Manuel Hilgert',
    'strasse': 'Birkenstraße 3',
    'ort': '85414 Kirchdorf',
    'land': 'Deutschland',
    'mail': 'hilman2@gmail.com',
}

# Pfad-Segmente je Sprache, damit die URLs in der jeweiligen Sprache lesbar sind.
LEGAL_PATHS = {
    'en': {'imprint': 'legal/', 'privacy': 'privacy/'},
    'de': {'imprint': 'de/impressum/', 'privacy': 'de/datenschutz/'},
    'fr': {'imprint': 'fr/mentions-legales/', 'privacy': 'fr/confidentialite/'},
    'es': {'imprint': 'es/aviso-legal/', 'privacy': 'es/privacidad/'},
}

LEGAL_NAV = {
    'en': {'imprint': 'Legal notice', 'privacy': 'Privacy'},
    'de': {'imprint': 'Impressum', 'privacy': 'Datenschutz'},
    'fr': {'imprint': 'Mentions légales', 'privacy': 'Confidentialité'},
    'es': {'imprint': 'Aviso legal', 'privacy': 'Privacidad'},
}

BACK = {'en': 'Back to the start page', 'de': 'Zurück zur Startseite',
        'fr': "Retour à la page d'accueil", 'es': 'Volver a la página de inicio'}

AUTHORITATIVE = {
    'en': 'This is a translation for convenience. The German version is the '
          'legally binding one.',
    'de': '',
    'fr': 'Ceci est une traduction de courtoisie. Seule la version allemande '
          'fait foi.',
    'es': 'Esta es una traducción de cortesía. Solo la versión alemana es '
          'jurídicamente vinculante.',
}

STAND = {'en': 'Last updated: 1 September 2026', 'de': 'Stand: 1. September 2026',
         'fr': 'Dernière mise à jour : 1er septembre 2026',
         'es': 'Última actualización: 1 de septiembre de 2026'}

IMPRINT = {
    'en': {
        'title': 'Legal notice | easy-ocpp',
        'h1': 'Legal notice',
        'blocks': [
            ('Information pursuant to section 5 DDG',
             ['{name}', '{strasse}', '{ort}', '{land}']),
            ('Contact', ['Email: {mail}']),
            ('Responsible for the content', ['{name}, address as above']),
        ],
        'links_h': 'Liability for links',
        'links_p': 'This site links to external websites. Their content is the '
                   'responsibility of their respective operators. At the time of '
                   'linking, no unlawful content was apparent. Permanent monitoring '
                   'without concrete evidence of an infringement is not reasonable; '
                   'links will be removed once such an infringement becomes known.',
    },
    'de': {
        'title': 'Impressum | easy-ocpp',
        'h1': 'Impressum',
        'blocks': [
            ('Angaben gemäß § 5 DDG',
             ['{name}', '{strasse}', '{ort}', '{land}']),
            ('Kontakt', ['E-Mail: {mail}']),
            ('Verantwortlich für den Inhalt', ['{name}, Anschrift wie oben']),
        ],
        'links_h': 'Haftung für Links',
        'links_p': 'Diese Seite verweist auf externe Websites. Für deren Inhalte '
                   'sind die jeweiligen Betreiber verantwortlich. Zum Zeitpunkt der '
                   'Verlinkung waren keine rechtswidrigen Inhalte erkennbar. Eine '
                   'dauerhafte Kontrolle ohne konkrete Anhaltspunkte für eine '
                   'Rechtsverletzung ist nicht zumutbar; bei Bekanntwerden einer '
                   'solchen wird der Link entfernt.',
    },
    'fr': {
        'title': 'Mentions légales | easy-ocpp',
        'h1': 'Mentions légales',
        'blocks': [
            ("Informations selon l'article 5 DDG (Allemagne)",
             ['{name}', '{strasse}', '{ort}', '{land}']),
            ('Contact', ['E-mail : {mail}']),
            ('Responsable du contenu', ['{name}, adresse ci-dessus']),
        ],
        'links_h': 'Responsabilité concernant les liens',
        'links_p': 'Ce site renvoie vers des sites externes. Leurs exploitants '
                   'respectifs sont responsables de leur contenu. Aucun contenu '
                   'illicite n\'était décelable au moment de la mise en lien. Un '
                   'contrôle permanent sans indice concret d\'une infraction n\'est '
                   'pas exigible ; tout lien sera retiré dès qu\'une infraction sera '
                   'portée à notre connaissance.',
    },
    'es': {
        'title': 'Aviso legal | easy-ocpp',
        'h1': 'Aviso legal',
        'blocks': [
            ('Información según el artículo 5 DDG (Alemania)',
             ['{name}', '{strasse}', '{ort}', '{land}']),
            ('Contacto', ['Correo electrónico: {mail}']),
            ('Responsable del contenido', ['{name}, dirección indicada arriba']),
        ],
        'links_h': 'Responsabilidad por los enlaces',
        'links_p': 'Este sitio enlaza a sitios web externos. De sus contenidos '
                   'responden sus respectivos operadores. En el momento de crear el '
                   'enlace no se apreciaban contenidos ilícitos. No es exigible una '
                   'supervisión permanente sin indicios concretos de una infracción; '
                   'el enlace se retirará en cuanto se tenga conocimiento de ella.',
    },
}

PRIVACY = {
    'en': {
        'title': 'Privacy | easy-ocpp',
        'h1': 'Privacy',
        'sections': [
            ('Controller',
             ['{name}, {strasse}, {ort}, {land}. Email: {mail}']),
            ('Hosting',
             ['This site is hosted on GitHub Pages by GitHub, Inc., 88 Colin P. '
              'Kelly Jr. Street, San Francisco, CA 94107, USA. When you open a '
              'page, your browser transmits data that GitHub records in server '
              'logs: your IP address, the date and time, the file requested, and '
              'information about your browser and operating system.',
              'This processing is necessary for the site to be delivered securely '
              'and reliably, and rests on Article 6(1)(f) GDPR. Data may be '
              'transferred to the United States. Details are set out in the GitHub '
              'Privacy Statement.']),
            ('What this site does not do',
             ['It sets no cookies and stores nothing in your browser. It loads no '
              'fonts, scripts, images or other files from third-party servers. '
              'There is no analytics, no tracking and no advertising. No consent '
              'banner is needed because there is nothing to consent to.']),
            ('Contacting us',
             ['If you write to the email address above, your message and the data '
              'it contains are processed in order to answer it, on the basis of '
              'Article 6(1)(f) GDPR. The data are deleted once they are no longer '
              'needed.']),
            ('Your rights',
             ['You have the right to obtain information about your data '
              '(Article 15 GDPR), to have it corrected (Article 16), erased '
              '(Article 17) or restricted (Article 18), the right to data '
              'portability (Article 20) and the right to object (Article 21).',
              'You may also lodge a complaint with a supervisory authority. The '
              'competent authority here is the Bavarian Data Protection Authority '
              '(Bayerisches Landesamt für Datenschutzaufsicht), Promenade 27, '
              '91522 Ansbach, Germany.']),
        ],
    },
    'de': {
        'title': 'Datenschutz | easy-ocpp',
        'h1': 'Datenschutzerklärung',
        'sections': [
            ('Verantwortlicher',
             ['{name}, {strasse}, {ort}, {land}. E-Mail: {mail}']),
            ('Hosting',
             ['Diese Seite liegt bei GitHub Pages, betrieben von GitHub, Inc., '
              '88 Colin P. Kelly Jr. Street, San Francisco, CA 94107, USA. Beim '
              'Aufruf einer Seite übermittelt Ihr Browser Daten, die GitHub in '
              'Server-Logs festhält: Ihre IP-Adresse, Datum und Uhrzeit, die '
              'angeforderte Datei sowie Angaben zu Browser und Betriebssystem.',
              'Diese Verarbeitung ist für die sichere und zuverlässige '
              'Auslieferung der Seite erforderlich und stützt sich auf Art. 6 '
              'Abs. 1 lit. f DSGVO. Dabei können Daten in die USA übermittelt '
              'werden. Einzelheiten stehen in der Datenschutzerklärung von '
              'GitHub.']),
            ('Was diese Seite nicht tut',
             ['Sie setzt keine Cookies und speichert nichts in Ihrem Browser. Sie '
              'lädt keine Schriften, Skripte, Bilder oder sonstigen Dateien von '
              'fremden Servern. Es gibt keine Reichweitenmessung, kein Tracking '
              'und keine Werbung. Ein Einwilligungsbanner ist deshalb nicht nötig, '
              'weil es nichts gibt, worin eingewilligt werden müsste.']),
            ('Kontaktaufnahme',
             ['Wenn Sie an die oben genannte E-Mail-Adresse schreiben, werden Ihre '
              'Nachricht und die darin enthaltenen Daten zur Bearbeitung Ihres '
              'Anliegens verarbeitet, gestützt auf Art. 6 Abs. 1 lit. f DSGVO. Die '
              'Daten werden gelöscht, sobald sie nicht mehr benötigt werden.']),
            ('Ihre Rechte',
             ['Sie haben das Recht auf Auskunft über Ihre Daten (Art. 15 DSGVO), '
              'auf Berichtigung (Art. 16), auf Löschung (Art. 17), auf '
              'Einschränkung der Verarbeitung (Art. 18), auf Datenübertragbarkeit '
              '(Art. 20) sowie das Recht auf Widerspruch (Art. 21).',
              'Außerdem können Sie sich bei einer Aufsichtsbehörde beschweren. '
              'Zuständig ist hier das Bayerische Landesamt für Datenschutzaufsicht, '
              'Promenade 27, 91522 Ansbach.']),
        ],
    },
    'fr': {
        'title': 'Confidentialité | easy-ocpp',
        'h1': 'Protection des données',
        'sections': [
            ('Responsable du traitement',
             ['{name}, {strasse}, {ort}, {land}. E-mail : {mail}']),
            ('Hébergement',
             ['Ce site est hébergé sur GitHub Pages, exploité par GitHub, Inc., '
              '88 Colin P. Kelly Jr. Street, San Francisco, CA 94107, États-Unis. '
              'À chaque consultation, votre navigateur transmet des données que '
              'GitHub consigne dans ses journaux de serveur : votre adresse IP, la '
              'date et l\'heure, le fichier demandé ainsi que des informations sur '
              'votre navigateur et votre système d\'exploitation.',
              'Ce traitement est nécessaire à une diffusion sûre et fiable du site '
              'et repose sur l\'article 6, paragraphe 1, point f, du RGPD. Des '
              'données peuvent être transférées aux États-Unis. Les détails '
              'figurent dans la déclaration de confidentialité de GitHub.']),
            ('Ce que ce site ne fait pas',
             ['Il ne dépose aucun cookie et n\'enregistre rien dans votre '
              'navigateur. Il ne charge ni polices, ni scripts, ni images, ni '
              'autres fichiers depuis des serveurs tiers. Il n\'y a ni mesure '
              'd\'audience, ni suivi, ni publicité. Aucune bannière de consentement '
              'n\'est donc nécessaire, faute d\'objet.']),
            ('Prise de contact',
             ['Si vous écrivez à l\'adresse électronique indiquée ci-dessus, votre '
              'message et les données qu\'il contient sont traités pour répondre à '
              'votre demande, sur le fondement de l\'article 6, paragraphe 1, '
              'point f, du RGPD. Les données sont effacées dès qu\'elles ne sont '
              'plus nécessaires.']),
            ('Vos droits',
             ['Vous disposez d\'un droit d\'accès à vos données (article 15 du '
              'RGPD), de rectification (article 16), d\'effacement (article 17), '
              'de limitation du traitement (article 18), à la portabilité '
              '(article 20) ainsi que d\'un droit d\'opposition (article 21).',
              'Vous pouvez également introduire une réclamation auprès d\'une '
              'autorité de contrôle. L\'autorité compétente ici est l\'Office '
              'bavarois de contrôle de la protection des données (Bayerisches '
              'Landesamt für Datenschutzaufsicht), Promenade 27, 91522 Ansbach, '
              'Allemagne.']),
        ],
    },
    'es': {
        'title': 'Privacidad | easy-ocpp',
        'h1': 'Protección de datos',
        'sections': [
            ('Responsable del tratamiento',
             ['{name}, {strasse}, {ort}, {land}. Correo electrónico: {mail}']),
            ('Alojamiento',
             ['Este sitio está alojado en GitHub Pages, operado por GitHub, Inc., '
              '88 Colin P. Kelly Jr. Street, San Francisco, CA 94107, EE. UU. Al '
              'abrir una página, su navegador transmite datos que GitHub registra '
              'en los registros del servidor: su dirección IP, la fecha y la hora, '
              'el archivo solicitado y datos sobre su navegador y sistema '
              'operativo.',
              'Este tratamiento es necesario para entregar el sitio de forma '
              'segura y fiable y se basa en el artículo 6, apartado 1, letra f, '
              'del RGPD. Es posible que se transfieran datos a los Estados Unidos. '
              'Los detalles constan en la declaración de privacidad de GitHub.']),
            ('Lo que este sitio no hace',
             ['No utiliza cookies ni guarda nada en su navegador. No carga '
              'tipografías, scripts, imágenes ni otros archivos desde servidores '
              'de terceros. No hay analítica, ni seguimiento, ni publicidad. Por '
              'eso no hace falta ningún banner de consentimiento: no hay nada que '
              'consentir.']),
            ('Contacto',
             ['Si escribe a la dirección de correo indicada arriba, su mensaje y '
              'los datos que contenga se tratarán para atender su solicitud, con '
              'base en el artículo 6, apartado 1, letra f, del RGPD. Los datos se '
              'suprimen en cuanto dejan de ser necesarios.']),
            ('Sus derechos',
             ['Tiene derecho a obtener información sobre sus datos (artículo 15 '
              'del RGPD), a su rectificación (artículo 16), a su supresión '
              '(artículo 17), a la limitación del tratamiento (artículo 18), a la '
              'portabilidad (artículo 20) y a oponerse al tratamiento '
              '(artículo 21).',
              'Además puede presentar una reclamación ante una autoridad de '
              'control. Aquí es competente la Autoridad Bávara de Supervisión de '
              'Protección de Datos (Bayerisches Landesamt für Datenschutzaufsicht), '
              'Promenade 27, 91522 Ansbach, Alemania.']),
        ],
    },
}

def esc(s):
    return (s.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
             .replace('"', '&quot;'))


def page(lang):
    c = C[lang]
    prefix = '../' if lang != 'en' else ''
    url = BASE + PATHS[lang]

    alternates = '\n'.join(
        '    <link rel="alternate" hreflang="%s" href="%s">' % (l, BASE + PATHS[l])
        for l in LANGS
    )
    alternates += '\n    <link rel="alternate" hreflang="x-default" href="%s">' % BASE

    langlinks = ' · '.join(
        ('<strong>%s</strong>' % LANG_NAMES[l]) if l == lang
        else '<a href="%s" hreflang="%s">%s</a>' % (BASE + PATHS[l], l, LANG_NAMES[l])
        for l in LANGS
    )

    software = {
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "easy-ocpp",
        "applicationCategory": "BusinessApplication",
        "applicationSubCategory": "Charging Station Management System (CSMS)",
        "operatingSystem": "Windows, Linux",
        "description": c['desc'],
        "url": url,
        "codeRepository": REPO,
        "programmingLanguage": "Rust",
        "license": "https://opensource.org/licenses/MIT",
        "isAccessibleForFree": True,
        "offers": {"@type": "Offer", "price": "0", "priceCurrency": "EUR"},
        "inLanguage": ["de", "en", "fr", "es"],
    }
    faq = {
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": [
            {"@type": "Question", "name": q,
             "acceptedAnswer": {"@type": "Answer", "text": a}}
            for q, a in c['faq']
        ],
    }

    pitch = '\n'.join(
        '        <article class="card">\n'
        '          <h3>%s</h3>\n'
        '          <p>%s</p>\n'
        '        </article>' % (esc(t), esc(b)) for t, b in c['pitch'])

    features = '\n'.join(
        '        <article class="feature">\n'
        '          <h3>%s</h3>\n'
        '          <p>%s</p>\n'
        '        </article>' % (esc(t), esc(b)) for t, b in c['features'])

    def cell(v):
        return '<td class="yes" aria-label="yes">&#10003;</td>' if v \
            else '<td class="no" aria-label="no">&ndash;</td>'

    roles = '\n'.join(
        '          <tr><th scope="row">%s</th>%s%s</tr>' % (esc(t), cell(a), cell(e))
        for t, a, e in c['roles'])

    compat = '\n'.join(
        '          <tr><th scope="row">%s</th><td>%s</td><td>%s</td></tr>'
        % (esc(v), esc(tr), esc(st)) for v, tr, st in c['compat'])

    faqs = '\n'.join(
        '        <details class="faq">\n'
        '          <summary><h3>%s</h3></summary>\n'
        '          <p>%s</p>\n'
        '        </details>' % (esc(q), esc(a)) for q, a in c['faq'])

    return """<!doctype html>
<html lang="%(lang)s">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>%(title)s</title>
    <meta name="description" content="%(desc)s">
    <link rel="canonical" href="%(url)s">
%(alternates)s
    <meta property="og:type" content="website">
    <meta property="og:site_name" content="easy-ocpp">
    <meta property="og:title" content="%(title)s">
    <meta property="og:description" content="%(desc)s">
    <meta property="og:url" content="%(url)s">
    <meta property="og:locale" content="%(locale)s">
    <meta property="og:image" content="%(ogimage)s">
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="%(title)s">
    <meta name="twitter:description" content="%(desc)s">
    <meta name="twitter:image" content="%(ogimage)s">
    <meta name="theme-color" content="#0b1220">
    <link rel="icon" href="data:image/svg+xml,%%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%%3E%%3Crect width='32' height='32' rx='7' fill='%%230b1220'/%%3E%%3Cpath d='M18 5 L9 18 h6 l-2 9 9-13 h-6 z' fill='%%2322d3ee'/%%3E%%3C/svg%%3E">
    <link rel="stylesheet" href="%(prefix)sassets/style.css">
    <script type="application/ld+json">%(software)s</script>
    <script type="application/ld+json">%(faqjson)s</script>
</head>
<body>
<a class="skip" href="#main">Skip to content</a>

<header class="site">
  <div class="wrap bar">
    <a class="brand" href="%(url)s">
      <svg viewBox="0 0 32 32" width="26" height="26" aria-hidden="true"><rect width="32" height="32" rx="7" fill="#0f172a"/><path d="M18 5 L9 18 h6 l-2 9 9-13 h-6 z" fill="#22d3ee"/></svg>
      easy-ocpp
    </a>
    <nav aria-label="Sections">
      <a href="#features">%(nav_features)s</a>
      <a href="#roles">%(nav_roles)s</a>
      <a href="#limits">%(nav_limits)s</a>
      <a href="#start">%(nav_start)s</a>
      <a href="#faq">%(nav_faq)s</a>
    </nav>
  </div>
</header>

<main id="main">

  <section class="hero">
    <div class="wrap">
      <p class="eyebrow">OCPP 1.6J &middot; 2.0.1 &middot; 1.5 SOAP</p>
      <h1>%(h1)s</h1>
      <p class="lede">%(lede)s</p>
      <p class="actions">
        <a class="btn primary" href="%(repo)s/releases/latest">%(cta_download)s</a>
        <a class="btn" href="%(repo)s">%(cta_source)s</a>
      </p>
    </div>
  </section>

  <section class="wrap grid-3">
%(pitch)s
  </section>

  <section id="features" class="wrap">
    <h2>%(features_h)s</h2>
    <div class="grid-4">
%(features)s
    </div>
  </section>

  <section id="roles" class="wrap">
    <h2>%(roles_h)s</h2>
    <p class="lede-2">%(roles_lede)s</p>
    <div class="tablewrap">
      <table>
        <thead><tr><th scope="col">%(rh0)s</th><th scope="col">%(rh1)s</th><th scope="col">%(rh2)s</th></tr></thead>
        <tbody>
%(roles)s
        </tbody>
      </table>
    </div>
    <p class="note">%(roles_note)s</p>
  </section>

  <section id="limits" class="wrap">
    <h2>%(limits_h)s</h2>
    <p>%(limits_body)s</p>
    <p class="note">%(limits_tech)s</p>
  </section>

  <section id="start" class="wrap">
    <h2>%(start_h)s</h2>
    <p>%(start_body)s</p>
    <p>%(start_build)s</p>
    <pre><code>git clone %(repo)s.git
cd easy-ocpp
cargo run --release</code></pre>

    <h3>%(connect_h)s</h3>
    <p>%(connect_body)s</p>
    <pre><code>ws://&lt;host&gt;:8080/ocpp/&lt;ChargePointId&gt;</code></pre>
    <p>%(connect_soap)s</p>
    <pre><code>POST http://&lt;host&gt;:8080/ocpp15</code></pre>
  </section>

  <section id="compat" class="wrap">
    <h2>%(compat_h)s</h2>
    <div class="tablewrap">
      <table>
        <thead><tr><th scope="col">%(ch0)s</th><th scope="col">%(ch1)s</th><th scope="col">%(ch2)s</th></tr></thead>
        <tbody>
%(compat)s
        </tbody>
      </table>
    </div>
  </section>

  <section id="faq" class="wrap">
    <h2>%(faq_h)s</h2>
%(faqs)s
  </section>

</main>

<footer class="site">
  <div class="wrap">
    <p class="langs"><span class="muted">%(footer_lang)s:</span> %(langlinks)s</p>
    <p class="muted">%(footer_license)s &middot; <a href="%(repo)s">GitHub</a></p>
    <p class="muted"><a href="%(imprint_url)s">%(imprint_label)s</a> &middot; <a href="%(privacy_url)s">%(privacy_label)s</a></p>
  </div>
</footer>
</body>
</html>
""" % {
        'lang': lang,
        'locale': {'en': 'en_GB', 'de': 'de_DE', 'fr': 'fr_FR', 'es': 'es_ES'}[lang],
        'title': esc(c['title']),
        'desc': esc(c['desc']),
        'url': url,
        'repo': REPO,
        'ogimage': OG_IMAGE,
        'prefix': prefix,
        'alternates': alternates,
        'software': json.dumps(software, ensure_ascii=False),
        'faqjson': json.dumps(faq, ensure_ascii=False),
        'h1': esc(c['h1']),
        'lede': esc(c['lede']),
        'cta_download': esc(c['cta_download']),
        'cta_source': esc(c['cta_source']),
        'nav_features': esc(c['nav_features']),
        'nav_roles': esc(c['nav_roles']),
        'nav_limits': esc(c['nav_limits']),
        'nav_start': esc(c['nav_start']),
        'nav_faq': esc(c['nav_faq']),
        'pitch': pitch,
        'features_h': esc(c['features_h']),
        'features': features,
        'roles_h': esc(c['roles_h']),
        'roles_lede': esc(c['roles_lede']),
        'rh0': esc(c['roles_head'][0]), 'rh1': esc(c['roles_head'][1]), 'rh2': esc(c['roles_head'][2]),
        'roles': roles,
        'roles_note': esc(c['roles_note']),
        'limits_h': esc(c['limits_h']),
        'limits_body': esc(c['limits_body']),
        'limits_tech': esc(c['limits_tech']),
        'start_h': esc(c['start_h']),
        'start_body': esc(c['start_body']),
        'start_build': esc(c['start_build']),
        'connect_h': esc(c['connect_h']),
        'connect_body': esc(c['connect_body']),
        'connect_soap': esc(c['connect_soap']),
        'compat_h': esc(c['compat_h']),
        'ch0': esc(c['compat_head'][0]), 'ch1': esc(c['compat_head'][1]), 'ch2': esc(c['compat_head'][2]),
        'compat': compat,
        'faq_h': esc(c['faq_h']),
        'faqs': faqs,
        'footer_lang': esc(c['footer_lang']),
        'footer_license': esc(c['footer_license']),
        'langlinks': langlinks,
        'imprint_url': BASE + LEGAL_PATHS[lang]['imprint'],
        'privacy_url': BASE + LEGAL_PATHS[lang]['privacy'],
        'imprint_label': esc(LEGAL_NAV[lang]['imprint']),
        'privacy_label': esc(LEGAL_NAV[lang]['privacy']),
    }



def legal_page(lang, kind):
    """Impressum (kind='imprint') oder Datenschutz (kind='privacy')."""
    url = BASE + LEGAL_PATHS[lang][kind]
    prefix = '../' * (LEGAL_PATHS[lang][kind].count('/'))
    c = IMPRINT[lang] if kind == 'imprint' else PRIVACY[lang]

    if kind == 'imprint':
        body = []
        for head, zeilen in c['blocks']:
            body.append('    <h2>%s</h2>' % esc(head))
            gefuellt = [esc(z.format(**ANBIETER)) for z in zeilen]
            body.append('    <p>%s</p>' % '<br>'.join(gefuellt))
        body.append('    <h2>%s</h2>' % esc(c['links_h']))
        body.append('    <p>%s</p>' % esc(c['links_p']))
    else:
        body = []
        for head, absaetze in c['sections']:
            body.append('    <h2>%s</h2>' % esc(head))
            for a in absaetze:
                body.append('    <p>%s</p>' % esc(a.format(**ANBIETER)))
    body = '\n'.join(body)

    hinweis = ''
    if AUTHORITATIVE.get(lang):
        hinweis = '  <p class="note">%s</p>\n' % esc(AUTHORITATIVE[lang])

    # Rechtsseiten gehoeren nicht in die Suchergebnisse zum Produkt, sind aber
    # ueber den Fuss jeder Seite mit einem Klick erreichbar.
    return """<!doctype html>
<html lang="%(lang)s">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>%(title)s</title>
    <meta name="robots" content="noindex, follow">
    <link rel="canonical" href="%(url)s">
    <meta name="theme-color" content="#0b1220">
    <link rel="stylesheet" href="%(prefix)sassets/style.css">
</head>
<body>
<header class="site">
  <div class="wrap bar">
    <a class="brand" href="%(home)s">
      <svg viewBox="0 0 32 32" width="26" height="26" aria-hidden="true"><rect width="32" height="32" rx="7" fill="#0f172a"/><path d="M18 5 L9 18 h6 l-2 9 9-13 h-6 z" fill="#22d3ee"/></svg>
      easy-ocpp
    </a>
  </div>
</header>

<main id="main" class="wrap legal">
  <h1>%(h1)s</h1>
%(hinweis)s%(body)s
  <p class="muted">%(stand)s</p>
  <p><a href="%(home)s">%(back)s</a></p>
</main>

<footer class="site">
  <div class="wrap">
    <p class="muted"><a href="%(imprint_url)s">%(imprint_label)s</a> &middot; <a href="%(privacy_url)s">%(privacy_label)s</a> &middot; <a href="%(repo)s">GitHub</a></p>
  </div>
</footer>
</body>
</html>
""" % {
        'lang': lang,
        'title': esc(c['title']),
        'url': url,
        'prefix': prefix,
        'home': BASE + PATHS[lang],
        'h1': esc(c['h1']),
        'hinweis': hinweis,
        'body': body,
        'stand': esc(STAND[lang]),
        'back': esc(BACK[lang]),
        'repo': REPO,
        'imprint_url': BASE + LEGAL_PATHS[lang]['imprint'],
        'privacy_url': BASE + LEGAL_PATHS[lang]['privacy'],
        'imprint_label': esc(LEGAL_NAV[lang]['imprint']),
        'privacy_label': esc(LEGAL_NAV[lang]['privacy']),
    }


CSS = """/* easy-ocpp landing page. Farben aus dem Dark-Cockpit-Design der Anwendung. */
:root {
  --bg: #0b1220;
  --surface: #131c2e;
  --surface-2: #1a2337;
  --border: #1f2a41;
  --border-strong: #2a3650;
  --fg: #e8edf6;
  --fg-2: #cbd5e1;
  --muted: #8894ab;
  --brand: #22d3ee;
  --brand-2: #06b6d4;
  --ok: #34d399;
  --radius: 12px;
  --font: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  --mono: ui-monospace, "JetBrains Mono", Consolas, monospace;
}

* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body {
  margin: 0;
  background: var(--bg);
  color: var(--fg);
  font-family: var(--font);
  font-size: 16px;
  line-height: 1.65;
  -webkit-font-smoothing: antialiased;
}
@media (prefers-reduced-motion: reduce) { html { scroll-behavior: auto; } }

.wrap { max-width: 1020px; margin: 0 auto; padding: 0 1.25rem; }
section.wrap { margin: 4.5rem auto; }

a { color: var(--brand); text-decoration: none; }
a:hover { text-decoration: underline; }
:focus-visible { outline: 2px solid var(--brand); outline-offset: 3px; border-radius: 4px; }

.skip {
  position: absolute; left: -9999px;
  background: var(--brand); color: #04222b; padding: .6rem 1rem; border-radius: 0 0 8px 0;
}
.skip:focus { left: 0; top: 0; z-index: 10; }

/* ---------- Kopf ---------- */
header.site {
  border-bottom: 1px solid var(--border);
  background: rgba(15, 23, 42, .85);
  backdrop-filter: blur(8px);
  position: sticky; top: 0; z-index: 5;
}
.bar { display: flex; align-items: center; gap: 1.5rem; height: 60px; }
.brand {
  display: flex; align-items: center; gap: .55rem;
  font-weight: 700; font-size: 1.05rem; color: var(--fg);
}
.brand:hover { text-decoration: none; }
header nav { display: flex; gap: 1.25rem; margin-left: auto; flex-wrap: wrap; }
header nav a { color: var(--fg-2); font-size: .92rem; }
@media (max-width: 720px) {
  .bar { height: auto; padding: .75rem 0; flex-wrap: wrap; gap: .75rem; }
  header nav { margin-left: 0; width: 100%; gap: 1rem; }
}

/* ---------- Hero ---------- */
.hero {
  padding: 5rem 0 4rem;
  background:
    radial-gradient(900px 420px at 15% -10%, rgba(34, 211, 238, .16), transparent 60%),
    radial-gradient(700px 380px at 90% 0%, rgba(167, 139, 250, .10), transparent 60%);
  border-bottom: 1px solid var(--border);
}
.eyebrow {
  margin: 0 0 1rem; color: var(--brand);
  font-family: var(--mono); font-size: .82rem; letter-spacing: .08em; text-transform: uppercase;
}
h1 {
  margin: 0 0 1.1rem;
  font-size: clamp(2rem, 5vw, 3.1rem);
  line-height: 1.12; letter-spacing: -.02em; font-weight: 800; max-width: 20ch;
}
.lede { font-size: 1.13rem; color: var(--fg-2); max-width: 68ch; margin: 0 0 2rem; }
.lede-2 { color: var(--fg-2); max-width: 72ch; }

.actions { display: flex; gap: .75rem; flex-wrap: wrap; margin: 0; }
.btn {
  display: inline-block; padding: .7rem 1.35rem; border-radius: var(--radius);
  border: 1px solid var(--border-strong); color: var(--fg); font-weight: 600; font-size: .97rem;
}
.btn:hover { text-decoration: none; border-color: var(--brand); }
.btn.primary { background: var(--brand); border-color: var(--brand); color: #04222b; }
.btn.primary:hover { background: var(--brand-2); border-color: var(--brand-2); }

/* ---------- Abschnitte ---------- */
h2 {
  font-size: clamp(1.5rem, 3vw, 2rem); letter-spacing: -.015em;
  margin: 0 0 1rem; font-weight: 750;
}
h3 { font-size: 1.02rem; margin: 0 0 .4rem; font-weight: 650; }
p { margin: 0 0 1rem; }
.note { color: var(--muted); font-size: .94rem; max-width: 72ch; }
.muted { color: var(--muted); }

.grid-3 { display: grid; gap: 1rem; grid-template-columns: repeat(3, 1fr); margin-top: -2.5rem; }
.grid-4 { display: grid; gap: 1rem; grid-template-columns: repeat(2, 1fr); }
@media (max-width: 860px) { .grid-3 { grid-template-columns: 1fr; margin-top: 2rem; } }
@media (max-width: 640px) { .grid-4 { grid-template-columns: 1fr; } }

.card {
  background: var(--surface); border: 1px solid var(--border);
  border-radius: var(--radius); padding: 1.25rem;
}
.card p { margin: 0; color: var(--fg-2); font-size: .95rem; }
.card h3 { color: var(--brand); }

.feature { border-left: 2px solid var(--border-strong); padding: .1rem 0 .1rem 1rem; }
.feature p { margin: 0; color: var(--fg-2); font-size: .95rem; }

/* ---------- Tabellen ---------- */
.tablewrap { overflow-x: auto; border: 1px solid var(--border); border-radius: var(--radius); }
table { border-collapse: collapse; width: 100%; min-width: 520px; font-size: .95rem; }
th, td { text-align: left; padding: .7rem .95rem; border-bottom: 1px solid var(--border); }
thead th { background: var(--surface-2); font-size: .82rem; text-transform: uppercase;
           letter-spacing: .05em; color: var(--muted); }
tbody tr:last-child th, tbody tr:last-child td { border-bottom: 0; }
tbody th { font-weight: 500; color: var(--fg-2); }
td.yes { color: var(--ok); font-weight: 700; }
td.no { color: var(--muted); }

/* ---------- Code ---------- */
pre {
  background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius);
  padding: 1rem 1.15rem; overflow-x: auto; margin: 0 0 1.25rem;
}
code { font-family: var(--mono); font-size: .89rem; color: var(--fg-2); }

/* ---------- FAQ ---------- */
.faq {
  border-bottom: 1px solid var(--border); padding: .35rem 0;
}
.faq summary {
  cursor: pointer; list-style: none; padding: .75rem 2rem .75rem 0; position: relative;
}
.faq summary::-webkit-details-marker { display: none; }
.faq summary h3 { display: inline; font-size: 1.05rem; color: var(--fg); }
.faq summary::after {
  content: "+"; position: absolute; right: .35rem; top: .6rem;
  color: var(--brand); font-size: 1.3rem; line-height: 1;
}
.faq[open] summary::after { content: "\\2212"; }
.faq p { color: var(--fg-2); max-width: 76ch; padding-bottom: .5rem; }

/* ---------- Fuss ---------- */
footer.site {
  border-top: 1px solid var(--border); margin-top: 5rem; padding: 2.5rem 0 3.5rem;
  font-size: .93rem;
}
footer .langs { margin-bottom: .5rem; }
footer p { margin: 0 0 .4rem; }

/* ---------- Rechtsseiten ---------- */
.legal { max-width: 760px; margin-top: 3rem; margin-bottom: 4rem; }
.legal h1 { font-size: clamp(1.7rem, 4vw, 2.4rem); margin-bottom: 1.5rem; }
.legal h2 { font-size: 1.1rem; margin: 2rem 0 .5rem; color: var(--brand); }
.legal p { color: var(--fg-2); }
"""


def main():
    os.makedirs('docs/assets', exist_ok=True)
    for l in LANGS:
        d = os.path.join('docs', PATHS[l].rstrip('/')) if PATHS[l] else 'docs'
        os.makedirs(d, exist_ok=True)
        p = os.path.join(d, 'index.html')
        io.open(p, 'w', encoding='utf-8', newline='\n').write(page(l))
        print('geschrieben:', p.replace('\\', '/'))

    for l in LANGS:
        for kind in ('imprint', 'privacy'):
            d = os.path.join('docs', *LEGAL_PATHS[l][kind].rstrip('/').split('/'))
            os.makedirs(d, exist_ok=True)
            p = os.path.join(d, 'index.html')
            io.open(p, 'w', encoding='utf-8', newline='\n').write(legal_page(l, kind))
            print('geschrieben:', p.replace('\\', '/'))

    io.open('docs/assets/style.css', 'w', encoding='utf-8', newline='\n').write(CSS)

    # Jekyll abschalten, die Seite ist fertiges HTML.
    io.open('docs/.nojekyll', 'w', encoding='utf-8', newline='\n').write('')

    sitemap = ['<?xml version="1.0" encoding="UTF-8"?>',
               '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"',
               '        xmlns:xhtml="http://www.w3.org/1999/xhtml">']
    for l in LANGS:
        sitemap.append('  <url>')
        sitemap.append('    <loc>%s</loc>' % (BASE + PATHS[l]))
        for alt in LANGS:
            sitemap.append('    <xhtml:link rel="alternate" hreflang="%s" href="%s"/>'
                           % (alt, BASE + PATHS[alt]))
        sitemap.append('    <xhtml:link rel="alternate" hreflang="x-default" href="%s"/>' % BASE)
        sitemap.append('    <changefreq>monthly</changefreq>')
        sitemap.append('  </url>')
    sitemap.append('</urlset>')
    io.open('docs/sitemap.xml', 'w', encoding='utf-8', newline='\n').write('\n'.join(sitemap) + '\n')

    io.open('docs/robots.txt', 'w', encoding='utf-8', newline='\n').write(
        'User-agent: *\nAllow: /\n\nSitemap: %ssitemap.xml\n' % BASE)

    print('geschrieben: docs/assets/style.css, docs/sitemap.xml, docs/robots.txt, docs/.nojekyll')


main()
