# easy-ocpp

🌐 [English](README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md)

[![CI](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml/badge.svg)](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hilman2/easy-ocpp)](https://github.com/hilman2/easy-ocpp/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**[Sitio del proyecto y documentación](https://hilman2.github.io/easy-ocpp/es/)**

Servidor **OCPP autoalojado (CSMS)** para estaciones de carga, pensado para **pymes con 1–10 cargadores**. Gestión de puntos de carga, tarjetas RFID, límites de carga e informes, sin suscripción en la nube.

- **Un solo binario, un solo archivo SQLite** – sin base de datos externa, sin message broker.
- **OCPP 1.6J** (completo) + **OCPP 2.0.1** (esqueleto WebSocket, BootNotification / TransactionEvent) + **OCPP 1.5 SOAP** (esqueleto para Boot/Heartbeat).
- **Interfaz web moderna** (Askama + htmx), cuentas locales con contraseñas Argon2.
- Las cuentas y las contraseñas están en el archivo SQLite. No hay conexión con un directorio ni con un inicio de sesión único.
- Funciona en **Windows** (prioridad) **y Linux**.

## Funcionalidades

| Área            | Estado |
|-----------------|--------|
| Inventario de cargadores + estado (online/offline, conectores, firmware) | ✅ |
| Gestión de tarjetas RFID, alta mediante «ventana de aprendizaje» (2 min, intercepción de Authorize) | ✅ |
| Asignación tarjeta → usuario; una tarjeta sin usuario es una tarjeta de invitado | ✅ |
| Desbloqueo remoto para invitados, imputación a una etiqueta de invitado | ✅ |
| Valores en vivo durante la carga (kWh cargados, potencia actual, SoC) | ✅ |
| Lista de transacciones, filtro por usuario | ✅ |
| Estadísticas por mes / trimestre / año | ✅ |
| Los usuarios **son** los empleados: tarjetas, cargas y límites cuelgan de la cuenta | ✅ |
| Límites por carga: kWh objetivo o temporizador, parada automática | ✅ |
| Valores predeterminados por persona, fijados por el empleado o por el admin | ✅ |
| Autoservicio del empleado: su carga en vivo, sus límites, parada por él mismo | ✅ |
| Cambio de contraseña, exigido por un administrador o hecho por el usuario | ✅ |
| Informe mensual enviado por correo a quienes hayan cargado | ✅ |
| Mensajes de los cargadores (estado y averías), 60 días, por carga y por cargador | ✅ |
| Interfaz multilingüe (Deutsch, English, Français, Español) | ✅ |

## Puesta en marcha

**Binarios listos para usar** (Windows x64, Linux x64) disponibles en
[Releases](https://github.com/hilman2/easy-ocpp/releases/latest). Descomprimir,
ejecutar `easy-ocpp.exe` o `easy-ocpp`, y listo (ver `INSTALL.es.md` en el paquete).

O compilarlo uno mismo:

```bash
# una sola vez: crear la configuración (opcional)
copy config.example.toml config.toml

# Build + inicio
cargo run --release
```

Después abrir la interfaz en <http://localhost:8080>. **Credenciales por defecto en el primer inicio:** `admin` / `admin` – cambie la contraseña de inmediato en «Usuarios».

¿Olvidó la contraseña de admin?

```bash
cargo run --release -- --reset-admin "nuevaContrasena123"
```

## Configurar un cargador

Los cargadores establecen una conexión WebSocket:

```
ws://<host>:8080/ocpp/<ChargePointId>
```

Los subprotocolos se negocian automáticamente (`ocpp1.6` u `ocpp2.0.1`).

Para dispositivos OCPP 1.5 más antiguos (SOAP):

```
POST http://<host>:8080/ocpp15
```

### Mediciones en vivo durante la carga

Al conectarse un cargador OCPP 1.6, el servidor lo configura automáticamente
para que durante una carga informe cada 30 segundos la lectura del contador, la
potencia y (si está disponible) el SoC (`MeterValueSampleInterval`,
`MeterValuesSampledData`). El intervalo puede ajustarse mediante `config.toml`;
`0` desactiva la configuración automática:

```toml
[ocpp]
meter_interval_s = 30
```

El panel y la página de detalle del cargador actualizan automáticamente las
cargas en curso cada 10 segundos (polling con htmx). Si un cargador no informa
la potencia, esta se deriva de las dos últimas lecturas del contador.

## Roles y permisos

Solo existe un tipo de persona: el **usuario**. Un empleado es un usuario con
`role = user`; sus tarjetas, sus cargas y sus límites cuelgan directamente de esa
cuenta. Ya no hay una ficha de empleado aparte.

| | Admin | Empleado |
|---|---|---|
| Panel, cargadores, tarjetas, gestión de usuarios | ✅ | – |
| Cargas de todas las personas | ✅ | – |
| Sus propias cargas (lista, CSV, estadísticas, PDF mensual) | ✅ | ✅ |
| Ver su carga en curso y detenerla | ✅ | ✅ |
| kWh objetivo / temporizador en su carga en curso | ✅ | ✅ |
| Valores predeterminados, los suyos | ✅ | ✅ |
| Valores predeterminados de cualquier empleado | ✅ | – |

Tras iniciar sesión, un empleado llega a **`/me`**, no al panel; la navegación
solo le muestra lo que realmente puede abrir. Las restricciones se aplican en el
servidor, no solo se ocultan en la interfaz.

Los empleados creados antes de este modelo de cuentas se importan **sin
contraseña**: sus cargas se registran, pero no pueden iniciar sesión hasta que un
administrador les asigne una en «Usuarios».

## Límites de carga

Una carga se detiene automáticamente en cuanto alcanza una **energía objetivo** o
un **temporizador**, lo que ocurra primero. Ambos son opcionales y se pueden
desactivar por separado.

- **Valores predeterminados por persona**: el empleado los fija en su propia
  página y el administrador en la ficha del usuario. Se aplican al iniciar cada
  carga, y el temporizador cuenta desde el comienzo de la carga.
- **En la carga en curso**: los valores se pueden cambiar mientras se carga. Allí
  el temporizador significa «detener dentro de N minutos», que es lo que uno
  quiere decir estando delante del cargador.

Un vigilante comprueba cada 15 segundos y envía `RemoteStopTransaction`. El
límite de energía se comprueba además en cuanto llegan nuevas mediciones, para no
seguir cargando hasta el siguiente ciclo. Si el cargador rechaza la parada, se
reintenta en el ciclo siguiente: la carga solo se da por resuelta cuando el
cargador la ha aceptado.

## Carga con excedentes y límite de red

El administrador puede activar **Carga con excedentes**, inicialmente
desactivada. Los chips normales usan solo excedentes; **Dirección / invitados**
tienen prioridad. **Red permitida** es un permiso independiente por chip, por
ejemplo para vehículos de empresa; nunca evita el límite del sitio. Los chips
de invitados, existentes y nuevos, tienen prioridad sin permiso de red.
Las prioridades iguales comparten potencia si caben sus mínimos; en caso
contrario algunos coches esperan a 0 W y reanudan al volver la potencia.

El límite predeterminado es **150000 W (150 kW)** con margen ajustable de
**10000 W**, para un objetivo de 140 kW de red en todo el edificio. Si otros
consumos ya superan ese objetivo, todos los coches se pausan. Revisar potencia
mínima/máxima, fases y unidad OCPP antes de activar. Los ajustes son por
conector; los límites en amperios suponen 230 V por fase y se redondean hacia abajo.

Un futuro adaptador del contador enviará lecturas sincronizadas cada pocos
segundos por HTTPS a `POST /api/energy/meter`, con
`Authorization: Bearer <meter_token>`. Configurar un token aleatorio de al menos
24 caracteres en `[energy]` de `config.toml` y reiniciar.
Ejemplo (sustituir la fecha por la hora real de medición):

```json
{"timestamp":"2026-10-15T10:00:00Z","grid_power_w":-5000,"charging_power_w":10000}
```

`grid_power_w` es la potencia del sitio: positiva para consumo, negativa para
vertido. `charging_power_w` es la **suma medida simultáneamente de todos los
conectores gestionados**, ya incluida en la potencia de red. No enviar
contadores de energía, consignas ni lecturas OCPP de momentos distintos.
El ejemplo representa 15 kW de excedente antes de los coches. El adaptador debe
sincronizar ambas medidas; aún no hay integración Modbus/MQTT ni de fabricante.
Las fechas caducadas, repetidas o futuras se rechazan. Las lecturas se guardan
en memoria y deben volver a recibirse tras reiniciar el servidor.

Sin lecturas recientes (15 segundos por defecto), se pausa la carga y se
bloquean nuevos inicios. Con lecturas recientes, una sesión autorizada puede
esperar a 0 W. Un perfil predeterminado a cero protege el inicio. Se confirman
primero las reducciones y se espera una nueva lectura para redistribuir.
Un perfil rechazado no libera potencia y provoca un intento de parada.
Los perfiles positivos vuelven a cero en 30 segundos sin renovación.
La preautorización local, la autorización sin conexión y la caché se desactivan
y permanecen así al desactivar la regulación. Desactivar elimina solo sus
propios perfiles y permite la carga normal. Antes de la primera activación no
se cambian ajustes del cargador.

**Requiere OCPP 1.6J**, Smart Charging, límites a cero y reloj comprobados.
Los otros protocolos se bloquean mientras está activa la regulación. Los
perfiles se han probado con simulador y deben validarse con los cargadores
reales. OCPP y un margen no garantizan un límite estricto de 150 kW ante picos
o fallos de comunicación; se necesita medición rápida y control local.

## Docker

```powershell
Copy-Item config.example.toml config.toml
docker compose up -d --build
```

Interfaz: <http://localhost:8080>. La base persiste en el volumen
`easy-ocpp-data`; `config.toml` se monta en solo lectura.
`docker compose down` conserva los datos; `docker compose down -v` los elimina.

### Detrás de un proxy inverso

`deploy/` contiene una configuración para un servidor en el que un solo Caddy
atiende varios servicios. `compose.yaml` conecta easy-ocpp a la red Docker del
proxy sin publicar ningún puerto, y `easy-ocpp.caddy.example` es el archivo de
sitio correspondiente.

Dos cosas importan para los cargadores. La mayoría habla OCPP sin TLS, así que
`ws://` por el puerto 80 tiene que llegar directamente a easy-ocpp: un cliente
OCPP no sigue una redirección a HTTPS. El ejemplo deja pasar por el puerto 80
solo `/ocpp/*` y `/ocpp15` y redirige todo lo demás. Además, recargar el proxy
cierra de inmediato los WebSockets abiertos si no se le da un plazo. De eso se
encarga `stream_close_delay`.

## Almacenamiento de datos

Todo se guarda en **un solo archivo SQLite** en `data/easy-ocpp.db` (modificable mediante `config.toml`). Las migraciones se encuentran en `migrations/` y se aplican automáticamente al iniciar.

### Comprobaciones de coherencia al recibir datos

- **Timestamps**: >24 h en el futuro o >10 años en el pasado se descartan – se recurre al reloj del servidor.
- **StartTransaction / StopTransaction**: idempotentes frente a repeticiones; los valores de contador decrecientes se corrigen.
- **StatusNotification**: UPSERT por (cargador, conector) – sin duplicados.
- **MeterValues**: los valores negativos se descartan, el SoC se valida entre 0 y 100 %, kWh → Wh normalizados.
- **Enrollment**: un tag recién capturado se asigna exactamente a una sesión abierta de ventana de aprendizaje.

## Estructura del proyecto

```
src/
  main.rs           – punto de entrada, runtime de Tokio, pool de SQLite
  config.rs         – configuración TOML
  db.rs             – bootstrap + helpers (Argon2, ajustes)
  error.rs          – AppError / IntoResponse
  auth/             – cookies de sesión + login local
  domain/           – modelos de datos (FromRow)
  ocpp/
    wire.rs         – parser de tramas JSON de OCPP
    hub.rs          – registro de todas las conexiones activas
    ocpp16.rs       – OCPP 1.6J (completo)
    ocpp20.rs       – OCPP 2.0.1 (bootstrap)
    soap15.rs       – endpoint OCPP 1.5 SOAP
    limits.rs       – vigilante de kWh objetivo / temporizador
  web/              – router axum, vistas Askama
templates/          – plantillas HTML (Askama)
static/             – CSS + shim de htmx (embebido vía rust-embed)
migrations/         – migraciones SQLite
```

## Licencia

MIT
