/*
===========================================================
PROYECTO: PANADERÍA WEB3
DESCRIPCIÓN GENERAL:

Este programa implementa una panadería descentralizada
utilizando Program Derived Addresses (PDAs).

Permite:
- Crear una panadería asociada a un propietario.
- Crear productos como cuentas independientes.
- Comprar productos reduciendo stock.
- Llevar un registro global del total vendido.


===========================================================
*/

use anchor_lang::prelude::*;


declare_id!("B3FSbwEyMDzC4nUSwacmKuaz3ez3EaRRSBiwz875zwUZ");

#[program]
pub mod panaderia_web3 {
    use super::*;

    // =====================================================
    // CREAR PANADERÍA
    // =====================================================
    // Crea una nueva cuenta PDA que representa una panadería.
    // Seeds utilizadas: ["panaderia", owner]
    // Solo puede existir una panadería por propietario.
    pub fn crear_panaderia(
        ctx: Context<CrearPanaderia>,
        nombre: String,
    ) -> Result<()> {

        // Validación: el nombre no debe estar vacío
        require!(!nombre.trim().is_empty(), Errores::NombreVacio);

        // Inicialización de la cuenta PDA
        ctx.accounts.panaderia.set_inner(Panaderia {
            owner: ctx.accounts.owner.key(),
            nombre,
        });

        msg!("Panadería creada correctamente");
        Ok(())
    }

    // =====================================================
    // INICIALIZAR VENTAS GLOBALES
    // =====================================================
    // Crea una cuenta PDA que almacenará el total vendido.
    // Seeds utilizadas: ["ventas", panaderia]
    // Solo el propietario puede inicializarla.
    pub fn inicializar_ventas(
        ctx: Context<InicializarVentas>,
    ) -> Result<()> {

        // Se inicia el contador global en cero
        ctx.accounts.ventas_globales.set_inner(VentasGlobales {
            panaderia: ctx.accounts.panaderia.key(),
            total_vendido: 0,
        });

        msg!("Cuenta de ventas globales creada");
        Ok(())
    }

    // =====================================================
    // AGREGAR PRODUCTO
    // =====================================================
    // Crea un nuevo producto como cuenta PDA independiente.
    // Seeds utilizadas:
    // ["producto", panaderia, nombre]
    // Solo el propietario puede crear productos.
    pub fn agregar_producto(
        ctx: Context<AgregarProducto>,
        nombre: String,
        precio: u64,
        stock: u16,
    ) -> Result<()> {

        // Validaciones de integridad
        require!(!nombre.trim().is_empty(), Errores::NombreVacio);
        require!(precio > 0, Errores::PrecioInvalido);
        require!(stock > 0, Errores::StockInvalido);

        // Inicialización del producto
        ctx.accounts.producto.set_inner(Producto {
            panaderia: ctx.accounts.panaderia.key(),
            nombre,
            precio,
            stock,
            disponible: true,
        });

        msg!("Producto creado correctamente");
        Ok(())
    }

    // =====================================================
    // COMPRAR PRODUCTO
    // =====================================================
    // Permite a un cliente comprar cierta cantidad.
    // - Reduce el stock.
    // - Marca como no disponible si llega a cero.
    // - Actualiza el total vendido global.
    pub fn comprar_producto(
        ctx: Context<ComprarProducto>,
        cantidad: u16,
    ) -> Result<()> {

        // Validación de cantidad válida
        require!(cantidad > 0, Errores::CantidadInvalida);

        let producto = &mut ctx.accounts.producto;

        // Verifica disponibilidad
        require!(producto.disponible, Errores::ProductoNoDisponible);

        // Verifica stock suficiente
        require!(producto.stock >= cantidad, Errores::StockInsuficiente);

        // Cálculo del total pagado
        let total_pagado = producto.precio * cantidad as u64;

        // Actualización de stock
        producto.stock -= cantidad;

        // Si se agota el stock, se marca como no disponible
        if producto.stock == 0 {
            producto.disponible = false;
        }

        // Se acumula el total vendido en la cuenta global
        ctx.accounts.ventas_globales.total_vendido += total_pagado;

        msg!("Compra realizada. Total pagado: {}", total_pagado);

        Ok(())
    }

    // =====================================================
    // VER TOTAL VENDIDO
    // =====================================================
    // Muestra el total acumulado almacenado en la cuenta
    // VentasGlobales.
    pub fn ver_total_vendido(
        ctx: Context<VerVentas>,
    ) -> Result<()> {

        msg!(
            "Total vendido acumulado: {}",
            ctx.accounts.ventas_globales.total_vendido
        );

        Ok(())
    }
}

// =====================================================
// CUENTAS (ESTADO EN BLOCKCHAIN)
// =====================================================

// Representa una panadería en la blockchain.
#[account]
#[derive(InitSpace)]
pub struct Panaderia {
    pub owner: Pubkey, // Propietario de la panadería

    #[max_len(60)]
    pub nombre: String, // Nombre comercial
}

// Representa un producto individual.
#[account]
#[derive(InitSpace)]
pub struct Producto {
    pub panaderia: Pubkey, // Relación con la panadería

    #[max_len(60)]
    pub nombre: String, // Nombre del producto

    pub precio: u64,     // Precio unitario
    pub stock: u16,      // Stock disponible
    pub disponible: bool,// Estado de disponibilidad
}

// Cuenta que almacena el total vendido acumulado.
#[account]
#[derive(InitSpace)]
pub struct VentasGlobales {
    pub panaderia: Pubkey, // Relación con la panadería
    pub total_vendido: u64,// Total acumulado de ventas
}

// =====================================================
// CONTEXTOS (VALIDACIÓN DE CUENTAS)
// =====================================================

// Contexto necesario para crear una panadería.
#[derive(Accounts)]
#[instruction(nombre: String)]
pub struct CrearPanaderia<'info> {

    #[account(mut)]
    pub owner: Signer<'info>, // Quien paga la creación

    #[account(
        init,
        payer = owner,
        space = 8 + Panaderia::INIT_SPACE,
        seeds = [b"panaderia", owner.key().as_ref()],
        bump
    )]
    pub panaderia: Account<'info, Panaderia>,

    pub system_program: Program<'info, System>,
}

// Contexto para inicializar la cuenta de ventas.
#[derive(Accounts)]
pub struct InicializarVentas<'info> {

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"panaderia", owner.key().as_ref()],
        bump,
        constraint = panaderia.owner == owner.key() @ Errores::NoEresElOwner
    )]
    pub panaderia: Account<'info, Panaderia>,

    #[account(
        init,
        payer = owner,
        space = 8 + VentasGlobales::INIT_SPACE,
        seeds = [b"ventas", panaderia.key().as_ref()],
        bump
    )]
    pub ventas_globales: Account<'info, VentasGlobales>,

    pub system_program: Program<'info, System>,
}

// Contexto para agregar productos.
#[derive(Accounts)]
#[instruction(nombre: String)]
pub struct AgregarProducto<'info> {

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"panaderia", owner.key().as_ref()],
        bump,
        constraint = panaderia.owner == owner.key() @ Errores::NoEresElOwner
    )]
    pub panaderia: Account<'info, Panaderia>,

    #[account(
        init,
        payer = owner,
        space = 8 + Producto::INIT_SPACE,
        seeds = [
            b"producto",
            panaderia.key().as_ref(),
            nombre.as_bytes()
        ],
        bump
    )]
    pub producto: Account<'info, Producto>,

    pub system_program: Program<'info, System>,
}

// Contexto para comprar productos.
#[derive(Accounts)]
pub struct ComprarProducto<'info> {

    #[account(mut)]
    pub cliente: Signer<'info>, // Cliente que ejecuta la compra

    pub panaderia: Account<'info, Panaderia>,

    #[account(mut)]
    pub producto: Account<'info, Producto>,

    #[account(
        mut,
        seeds = [b"ventas", panaderia.key().as_ref()],
        bump
    )]
    pub ventas_globales: Account<'info, VentasGlobales>,
}

// Contexto para consultar ventas.
#[derive(Accounts)]
pub struct VerVentas<'info> {

    #[account(
        seeds = [b"ventas", panaderia.key().as_ref()],
        bump
    )]
    pub ventas_globales: Account<'info, VentasGlobales>,

    pub panaderia: Account<'info, Panaderia>,
}

// =====================================================
// ERRORES PERSONALIZADOS
// =====================================================

// Errores definidos para validaciones del programa.
#[error_code]
pub enum Errores {

    #[msg("No eres el propietario")]
    NoEresElOwner,

    #[msg("Nombre vacío")]
    NombreVacio,

    #[msg("Precio inválido")]
    PrecioInvalido,

    #[msg("Stock inválido")]
    StockInvalido,

    #[msg("Cantidad inválida")]
    CantidadInvalida,

    #[msg("Stock insuficiente")]
    StockInsuficiente,

    #[msg("Producto no disponible")]
    ProductoNoDisponible,
}
