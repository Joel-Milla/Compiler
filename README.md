Para correr el codigo, se necesita tener instalada la version mas nueva de rust. El codigo principal se encuentra en src/main. El codigo fue realizado solamente apoyandose con la documentacion de rust y usando las librerias externas: Vector,VecDeque, y BTreeMap.

El codigo fuente se encuentra en:
https://github.com/Joel-Milla/Compiler

Comando para correr el codigo:

```(bash)
cargo run
```

Las pruebas realizadas son:

- Stack: Se crea un stack = (32 y 2). Luego se hace pop y se verifica que sale el 2 (el último que entro). Luego se revisa el top y se confirma que es 32. Con esto se valida que funciona como LIFO.
- Queue: Se crea una queue (32 y 64). Luego se hace dequeue y se verifica que sale el 32 (el primero que entro). Luego se revisa el front y se confirma que es 64. Con esto se valida que funciona como FIFO.
- Map ordenado: se insertan tres películas con ids 0, 1 y 2. Se busca el id 4 que no existe y se confirma que no lo encuentra. Se elimina la película con id 2 y después se buscan los ids 1 y 2, validando que el 1 si aparece y el 2 ya no porque se elimino.
