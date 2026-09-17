# 02.2 — Expresiones y precedencia

> **Status:** Abierto  
> **Summary:** La precedencia y la asociatividad de las expresiones deben ser inequívocas antes de que M2 pueda considerarse lo bastante estable para las herramientas posteriores.

La precedencia no es una minucia de formato. `a + b * c` debe tener un único
significado en toda implementación conforme.

Un diseño de parser debería explicitar estas propiedades:

- las operaciones postfijas se unen con más fuerza que los operadores unarios;
- la multiplicación y la división se unen con más fuerza que la suma y la resta;
- la comparación y la igualdad tienen un comportamiento de encadenamiento definido;
- los operadores booleanos tienen una precedencia fija;
- la asignación, si se introduce, debe definir su asociatividad;
- las futuras formas `ask`, `verify` o `delegate` deben declarar si son expresiones
  primarias, expresiones prefijas o declaraciones.

El formateador debería imprimir suficiente sintaxis para preservar el significado
sin depender de la intuición humana.
