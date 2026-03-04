
/*

PROYECTO: PANADERÍA WEB3

DESCRIPCIÓN GENERAL:

Este programa simula una panadería descentralizada en la
blockchain de Solana.

Permite:

1. Crear una panadería asociada a un propietario.
2. Crear productos como cuentas independientes (PDA).
3. Comprar productos reduciendo stock.
4. Registrar el total vendido en una cuenta global.


ARQUITECTURA:

- Panaderia (PDA)
  Seeds: ["panaderia", owner]

- Producto (PDA por producto)
  Seeds: ["producto", panaderia, nombre]

- VentasGlobales (PDA única por panadería)
  Seeds: ["ventas", panaderia]

FLUJO DE USO:

1) El owner crea la panadería.
2) El owner inicializa la cuenta de ventas globales.
3) El owner agrega productos.
4) Un cliente compra productos.
5) Se actualiza el total vendido automáticamente.


OBJETIVO:

Demostrar uso de:
- Program Derived Addresses (PDA)
- Validaciones con require!
- Manejo de estado en Solana
- Arquitectura tipo CRUD
- Relaciones entre cuentas

===========================================================
*/
use anchor_lang::prelude::*;

declare_id!("B3FSbwEyMDzC4nUSwacmKuaz3ez3EaRRSBiwz875zwUZ");

#[program]
pub mod panaderia_web3 {
    use super::*;

    // =============================
    // CREAR PANADERIA
    // =============================
    pub fn crear_panaderia(
        ctx: Context<CrearPanaderia>,
        nombre: String,
    ) -> Result<()> {

        require!(!nombre.trim().is_empty(), Errores::NombreVacio);

        ctx.accounts.panaderia.set_inner(Panaderia {
            owner: ctx.accounts.owner.key(),
            nombre,
        });

        msg!("Panadería creada correctamente");
        Ok(())
    }

    // =============================
    // INICIALIZAR VENTAS GLOBALES
    // =============================
    pub fn inicializar_ventas(
        ctx: Context<InicializarVentas>,
    ) -> Result<()> {

        ctx.accounts.ventas_globales.set_inner(VentasGlobales {
            panaderia: ctx.accounts.panaderia.key(),
            total_vendido: 0,
        });

        msg!("Cuenta de ventas globales creada");
        Ok(())
    }

    // =============================
    // AGREGAR PRODUCTO
    // =============================
    pub fn agregar_producto(
        ctx: Context<AgregarProducto>,
        nombre: String,
        precio: u64,
        stock: u16,
    ) -> Result<()> {

        require!(!nombre.trim().is_empty(), Errores::NombreVacio);
        require!(precio > 0, Errores::PrecioInvalido);
        require!(stock > 0, Errores::StockInvalido);

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

    // =============================
    // COMPRAR PRODUCTO
    // =============================
    pub fn comprar_producto(
        ctx: Context<ComprarProducto>,
        cantidad: u16,
    ) -> Result<()> {

        require!(cantidad > 0, Errores::CantidadInvalida);

        let producto = &mut ctx.accounts.producto;

        require!(producto.disponible, Errores::ProductoNoDisponible);
        require!(producto.stock >= cantidad, Errores::StockInsuficiente);

        let total_pagado = producto.precio * cantidad as u64;

        producto.stock -= cantidad;

        if producto.stock == 0 {
            producto.disponible = false;
        }

        ctx.accounts.ventas_globales.total_vendido += total_pagado;

        msg!("Compra realizada. Total pagado: {}", total_pagado);

        Ok(())
    }

    // =============================
    // VER TOTAL VENDIDO
    // =============================
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

// =============================
// CUENTAS
// =============================

#[account]
#[derive(InitSpace)]
pub struct Panaderia {
    pub owner: Pubkey,

    #[max_len(60)]
    pub nombre: String,
}

#[account]
#[derive(InitSpace)]
pub struct Producto {
    pub panaderia: Pubkey,

    #[max_len(60)]
    pub nombre: String,

    pub precio: u64,
    pub stock: u16,
    pub disponible: bool,
}

#[account]
#[derive(InitSpace)]
pub struct VentasGlobales {
    pub panaderia: Pubkey,
    pub total_vendido: u64,
}

// =============================
// CONTEXTOS
// =============================

#[derive(Accounts)]
#[instruction(nombre: String)]
pub struct CrearPanaderia<'info> {

    #[account(mut)]
    pub owner: Signer<'info>,

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
        seeds = [
            b"ventas",
            panaderia.key().as_ref()
        ],
        bump
    )]
    pub ventas_globales: Account<'info, VentasGlobales>,

    pub system_program: Program<'info, System>,
}

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

#[derive(Accounts)]
pub struct ComprarProducto<'info> {

    #[account(mut)]
    pub cliente: Signer<'info>,

    pub panaderia: Account<'info, Panaderia>,

    #[account(mut)]
    pub producto: Account<'info, Producto>,

    #[account(
        mut,
        seeds = [
            b"ventas",
            panaderia.key().as_ref()
        ],
        bump
    )]
    pub ventas_globales: Account<'info, VentasGlobales>,
}

#[derive(Accounts)]
pub struct VerVentas<'info> {

    #[account(
        seeds = [b"ventas", panaderia.key().as_ref()],
        bump
    )]
    pub ventas_globales: Account<'info, VentasGlobales>,

    pub panaderia: Account<'info, Panaderia>,
}

// =============================
// ERRORES
// =============================

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
