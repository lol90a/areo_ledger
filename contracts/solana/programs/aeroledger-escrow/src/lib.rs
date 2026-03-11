use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("");

#[program]
pub mod aeroledger_escrow {
    use super::*;

    pub fn create_order(ctx: Context<CreateOrder>, order_id: String, amount: u64) -> Result<()> {
        let order = &mut ctx.accounts.order;
        order.buyer = ctx.accounts.buyer.key();
        order.amount = amount;
        order.status = OrderStatus::Pending;
        order.created_at = Clock::get()?.unix_timestamp;
        order.order_id = order_id;
        order.bump = ctx.bumps.order;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer_token_account.to_account_info(),
                    to: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn confirm_order(ctx: Context<ConfirmOrder>) -> Result<()> {
        let order = &mut ctx.accounts.order;
        require!(order.status == OrderStatus::Pending, ErrorCode::InvalidStatus);
        order.status = OrderStatus::Confirmed;
        Ok(())
    }

    pub fn complete_order(ctx: Context<CompleteOrder>) -> Result<()> {
        let order = &mut ctx.accounts.order;
        require!(order.status == OrderStatus::Confirmed, ErrorCode::InvalidStatus);
        order.status = OrderStatus::Completed;

        let seeds = &[b"order", order.buyer.as_ref(), order.order_id.as_bytes(), &[order.bump]];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.admin_token_account.to_account_info(),
                    authority: ctx.accounts.order.to_account_info(),
                },
                signer,
            ),
            order.amount,
        )?;

        Ok(())
    }

    pub fn refund_order(ctx: Context<RefundOrder>) -> Result<()> {
        let order = &mut ctx.accounts.order;
        require!(order.status == OrderStatus::Pending || order.status == OrderStatus::Confirmed, ErrorCode::CannotRefund);
        
        let current_time = Clock::get()?.unix_timestamp;
        require!(current_time <= order.created_at + 86400, ErrorCode::RefundWindowExpired);

        order.status = OrderStatus::Refunded;

        let seeds = &[b"order", order.buyer.as_ref(), order.order_id.as_bytes(), &[order.bump]];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    authority: ctx.accounts.order.to_account_info(),
                },
                signer,
            ),
            order.amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(order_id: String)]
pub struct CreateOrder<'info> {
    #[account(init, payer = buyer, space = 8 + 32 + 8 + 1 + 8 + 54 + 1, seeds = [b"order", buyer.key().as_ref(), order_id.as_bytes()], bump)]
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ConfirmOrder<'info> {
    #[account(mut)]
    pub order: Account<'info, Order>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct CompleteOrder<'info> {
    #[account(mut)]
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin_token_account: Account<'info, TokenAccount>,
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RefundOrder<'info> {
    #[account(mut)]
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Order {
    pub buyer: Pubkey,
    pub amount: u64,
    pub status: OrderStatus,
    pub created_at: i64,
    pub order_id: String,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Completed,
    Refunded,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid order status")]
    InvalidStatus,
    #[msg("Cannot refund this order")]
    CannotRefund,
    #[msg("Refund window has expired")]
    RefundWindowExpired,
}
