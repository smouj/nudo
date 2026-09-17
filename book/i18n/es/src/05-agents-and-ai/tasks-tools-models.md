# 05.2 — Tareas, herramientas y modelos

> **Status:** Propuesto  
> **Summary:** NUDO separa los objetivos, las herramientas con efectos y los proveedores de modelos para que la autonomía pueda acotarse sin incrustar un proveedor en el lenguaje.


{{#include ../diagrams/interfaces.svg}}

*Las cuatro declaraciones y qué restringe a cada una.*

## Tarea

Una tarea es una unidad de trabajo autónomo con objetivo, resultado esperado, límites
y trazabilidad. A diferencia de una llamada a función normal, se espera que una tarea
produzca evidencia sobre cómo se realizó el trabajo.

## Herramienta

Una herramienta conecta el código con una capacidad externa: red, sistema de
archivos, shell, Git u otro servicio. Por eso la invocación de herramientas tiene
efectos y está condicionada por capacidades.

## Modelo

Un modelo es una interfaz de runtime neutral respecto al proveedor. Los campos de la
API de un proveedor y la autenticación no deberían cambiar el significado de un
programa NUDO.
