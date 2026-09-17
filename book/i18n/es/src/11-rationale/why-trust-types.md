# 11.2 — Por qué la confianza forma parte del sistema de tipos

> **Status:** Justificación propuesta (PROPOSED RATIONALE)  
> **Summary:** El estado de confianza es lo bastante importante como para ser visible en las interfaces, porque olvidar un paso de verificación es una clase de error de programación, y no solo una cuestión de registro.

Quien revisa debería poder inspeccionar la firma de una función y distinguir un valor
que vino de un modelo de otro que pasó por un verificador explícito.

Esa distinción también crea un lugar para las herramientas: los diagnósticos pueden
explicar la frontera que falta, los IDE pueden visualizar el flujo de confianza y las
auditorías pueden buscar valores verificados cuya procedencia esté incompleta.
