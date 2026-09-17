# 01.3 — Local-first y agnóstico del proveedor

> **Status:** Especificado (SPECIFIED)  
> **Summary:** Los modelos locales, las herramientas locales y la ejecución sin conexión se conciben como modos de primera clase, no como modos degradados.

Un nombre de proveedor no debería aparecer en el lenguaje central solo porque ese
proveedor sea popular hoy. Los proveedores cambian; la semántica del lenguaje no
debería cambiar.

Por eso NUDO separa una abstracción de modelo de los adaptadores propios de cada
proveedor. Un modelo local debería poder satisfacer la misma interfaz de runtime que
un modelo remoto, sujeto a capacidades y políticas.

Local-first también mejora la capacidad de prueba: los arneses de prueba
deterministas y los datos de prueba con modelos locales pueden ejercitar un programa
sin requerir infraestructura externa de pago.
