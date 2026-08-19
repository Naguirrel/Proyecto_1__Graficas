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
└── wall5.png
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
