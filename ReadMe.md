# Proyecto 1 - Raycasting

Proyecto universitario de Graficas por Computadora. El juego carga laberintos
desde archivos `.txt`, permite escoger entre tres niveles, recorrerlos en vista
2D o 3D, y termina cuando el jugador llega a la meta `g`.

## Tecnologia

- Rust
- minifb

## Funcionalidades

- Maze cargado desde archivo.
- Tres niveles cargados desde archivos `.txt`.
- Selector de nivel en la pantalla de bienvenida.
- Vista 2D de depuracion.
- Vista 3D estilo raycaster.
- Pantalla de bienvenida con estetica del nivel.
- Minimapa en la esquina durante la vista 3D.
- Raycasting con FOV.
- Movimiento con delta time.
- Contador de FPS visible durante gameplay.
- Colisiones y wall sliding.
- Fish-eye correction.
- Salto.
- Sistema de carga de texturas en memoria preparado para el renderer 3D.
- Cinco texturas normales de pared.
- Textura morada/negra reservada para las paredes de la meta.
- Mapa sin repetir el mismo simbolo de muro en celdas consecutivas.
- Bresenham para rasterizacion de lineas, rayos y stakes.
- Meta y condicion de victoria.
- Pantalla de exito.
- Musica de fondo en loop durante cada nivel.
- Musica temporal de poder durante 30 segundos al recoger comida.

## Auditoria contra la rubrica

La siguiente tabla compara la rubrica entregada para el proyecto con la
implementacion actual. El puntaje final lo decide el docente y tiene un maximo
de 100 puntos; por eso no se deben sumar los puntos de la tabla como si fueran
un resultado final.

| Criterio de la rubrica | Maximo | Evidencia en el proyecto | Estado |
| --- | ---: | --- | --- |
| Multiple hardware: soporte a mando/control | 20 | `src/gamepad.rs` usa `gilrs` para gamepads y el juego integra movimiento, menus, salto y cambio de vista. | Cumplido |
| Estetica del nivel | 30 | Pantallas tematicas, colores, texturas de paredes, piso, comida y fantasmas. La valoracion es subjetiva. | Implementado; sujeto a evaluacion |
| Mantener al menos 15 FPS | 15 | FPS visible en pantalla y benchmark de renderizado en `src/render.rs`. Debe validarse durante la demostracion en el equipo objetivo. | Instrumentado; validar en ejecucion |
| Camara con movimiento hacia adelante/atras y rotacion | 20 | `src/input.rs` implementa avance, retroceso y rotacion con teclado y mando; el raycaster usa la posicion y el angulo del jugador. | Cumplido |
| Movimiento horizontal de camara con mouse | 10 | `MouseLook` lee el delta horizontal del mouse y modifica el angulo del jugador. | Cumplido |
| Minimapa en una esquina | 10 | `render_minimap` lo dibuja en la esquina inferior derecha durante la vista 3D. | Cumplido |
| Musica de fondo | 5 | `src/audio.rs` reproduce en loop `assets/audios/cfondo.mp3`. | Cumplido |
| Extra por musica de Taylor Swift | 5 | Existe `assets/audios/poder_ts.mp3`, pero el contenido del MP3 no puede verificarse desde el codigo. | Pendiente de confirmar |
| Efectos de sonido | 10 | Solo se encontraron pistas de musica; no hay reproduccion de efectos asociada a eventos del juego. | No implementado |
| Al menos una animacion de sprite | 20 | Hay sprites de comida y fantasmas; los fantasmas se mueven y tienen variantes visuales, pero no hay cambio de cuadros por tiempo. | Parcial; validar con el docente |
| Pantalla de bienvenida | 5 | `render_welcome_screen` muestra el menu inicial y permite iniciar o salir. | Cumplido |
| Seleccion de multiples niveles | 10 | Se cargan `maze.txt`, `maze_2.txt` y `maze_3.txt`, y se pueden seleccionar desde el menu. | Cumplido |
| Pantalla de exito | 10 | `render_victory_screen` muestra `GANASTE` y permite reiniciar, avanzar o volver al menu. | Cumplido |

### Resultado de la auditoria

Hay evidencia directa de los criterios de control, estetica, rendimiento
instrumentado, camara, mouse, minimapa, musica, bienvenida, multiples niveles
y victoria. Los pendientes antes de entregar son los efectos de sonido, una
animacion de sprite por cuadros y confirmar si `poder_ts.mp3` cumple el extra de
Taylor Swift. La suma aritmetica de los maximos no representa la nota final,
porque la rubrica indica que no hay puntos extra y que la nota se limita a 100.

## Ejecutar

```bash
cargo run
```

## Controles

Antes de iniciar:

```text
A / D   Cambiar nivel
Left / Right
        Cambiar nivel
ENTER   Iniciar
ESC     Salir
```

Durante el juego:

```text
W       Avanzar
S       Retroceder
A       Girar izquierda
D       Girar derecha
Mouse   Girar camara horizontalmente
SPACE   Saltar
TAB     Alternar 2D / 3D
ESC     Salir
```

Control PS5 / DualSense:

```text
Stick izquierdo / D-Pad
        Avanzar, retroceder y navegar niveles
Stick derecho
        Girar camara horizontalmente
L1 / R1 Girar camara horizontalmente
X       Iniciar, saltar y reiniciar
Triangulo / Share / R3
        Alternar 2D / 3D
Circulo / Cuadrado
        Cambiar nivel en bienvenida
Options Iniciar y reiniciar
```

Despues de ganar:

```text
R       Reiniciar
ESC     Salir
```

## Pruebas

```bash
cargo test
```

## Estructura del Proyecto

- `src/main.rs`: ciclo principal, configuracion de la ventana y cambio de modo
  de renderizado
- `src/maze.rs`: carga, validacion, consulta de celdas y reglas de movimiento
  del laberinto
- `src/player.rs`: posicion, angulo, campo de vision y estado de salto del
  jugador
- `src/input.rs`: manejo del teclado y validacion de colisiones al moverse
- `src/caster.rs`: logica de raycasting para rayos 2D e impactos de paredes 3D
- `src/render.rs`: renderizado del mapa 2D, proyeccion 3D y asignacion de
  colores
- `src/audio.rs`: busqueda y reproduccion en loop de musica de fondo y poder
- `src/texture.rs`: carga de imagenes PNG, almacenamiento de texturas en
  memoria y acceso individual a pixeles
- `src/framebuffer.rs`: abstraccion del buffer de pixeles usado por el renderer
- `src/line.rs`: helper para dibujar lineas
- `assets/`: imagenes PNG que se cargan como texturas de pared
- `maze.txt`: primer nivel editable
- `maze_2.txt`: segundo nivel editable
- `maze_3.txt`: tercer nivel editable

## Niveles

El juego carga estos archivos al iniciar:

```text
maze.txt
maze_2.txt
maze_3.txt
```

En la pantalla de bienvenida se puede cambiar el nivel seleccionado con `A`/`D`
o con las flechas izquierda/derecha. Al presionar `ENTER`, el jugador inicia en
el `p` del nivel seleccionado.

## Texturas

Las paredes del renderer 3D se texturizan con imagenes PNG cargadas desde
`assets/`. `TextureManager` decodifica los placeholders una sola vez y mantiene
las imagenes en memoria como pixeles `u32` compatibles con el framebuffer.
La coordenada `tx` selecciona la columna horizontal segun el impacto del rayo y
`ty` selecciona la fila vertical de cada pixel visible de la stake. Bresenham
sigue rasterizando cada stake mediante una variante con color por pixel.

Estructura esperada:

```text
assets/
├── wall1.png
├── wall2.png
├── wall3.png
├── wall4.png
├── wall5.png
└── audios/
    ├── cfondo.mp3
    └── poder_ts.mp3
```

Mapeo preparado:

```text
# -> wall1.png
+ -> wall2.png
% -> wall3.png
@ -> wall4.png
& -> wall5.png
! -> wall5.png
```

## Audio

El juego busca audios MP3 dentro de `assets/audios/`.

- La musica de fondo usa el archivo cuyo nombre contiene `c` y `fondo`.
- La musica de poder usa el archivo cuyo nombre termina en `ts.mp3`.
- La musica de fondo se repite mientras se juega un nivel.
- Al recoger comida, la musica de poder reemplaza la de fondo durante los 30
  segundos del poder. Al terminar el poder, vuelve la musica de fondo.

## Formato del Laberinto

Cada laberinto es un archivo de texto rectangular. Debe contener exactamente un
inicio del jugador (`p`) y una meta (`g`). El borde exterior debe estar formado
por paredes.

Caracteres reconocidos:

- `#`, `+`, `%`, `@`, `&`: paredes normales con texturas diferentes
- `!`: paredes moradas/negras reservadas para la meta
- espacio: piso transitable
- `p`: inicio del jugador
- `g`: meta
