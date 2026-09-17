# 03.3 — Parser, sintaxis sin pérdidas y AST

> **Status:** Planificado (PLANNED)  
> **Summary:** M2 debería convertir el flujo de tokens en una estructura sintáctica sin pérdidas, recuperarse de entradas malformadas y exponer un AST tipado para las fases semánticas posteriores.

## Por qué primero una sintaxis sin pérdidas

Un árbol sin pérdidas conserva los elementos accesorios y cada byte del código
fuente. Esa propiedad importa para un formateador, para las herramientas de edición y
para los agentes automatizados que necesitan modificar código sin destruir en
silencio comentarios o formato.

## Recuperación de errores

Un parser útil no se detiene en el primer token que falta. Los puntos de recuperación
deberían incluir los límites de elemento, los puntos y coma y los delimitadores de
cierre, de modo que un solo error no convierta el resto del archivo en ruido.

## AST tipado

El AST debería ofrecer a las fases posteriores formas semánticas (`Function`, `Call`,
`Type`) sin obligarlas a conocer los detalles crudos de los nodos sintácticos. Las
conversiones de la sintaxis al AST deben preservar los spans.
