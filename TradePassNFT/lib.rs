#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, IntoVal, Map, String};

// Định nghĩa cấu trúc dữ liệu cho một TradePass NFT
#[contracttype]
#[derive(Clone)]
pub struct TradePass {
    pub owner: Address,       // Chủ sở hữu thẻ
    pub is_active: bool,     // Trạng thái thẻ: hoạt động hay bị khóa
    pub level: u32,          // Cấp độ giao dịch (ví dụ: cấp 1, cấp 2)
}

// Định nghĩa các loại lưu trữ (Storage Key)
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,                  // Lưu địa chỉ Admin
    Pass(Address),          // Lưu thẻ TradePass gắn với địa chỉ người dùng
}

#[contract]
pub struct TradePassContract;

#[contractimpl]
impl TradePassContract {

    // --- Hàm khởi tạo (Chỉ gọi 1 lần duy nhất) ---
    // Gán tài khoản hiện tại làm Admin đầu tiên.
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    // --- Hàm Mint (Cấp thẻ) ---
    // Chỉ Admin mới được quyền gọi hàm này.
    pub fn mint(env: Env, admin_auth: Address, user: Address, level: u32) {
        // 1. Xác thực quyền Admin
        let current_admin = env.storage().instance().get(&DataKey::Admin).unwrap_or_else(|| panic!("Not initialized"));
        if admin_auth != current_admin {
            panic!("Not authorized: Only Admin can mint");
        }
        admin_auth.require_auth(); // Yêu cầu ký xác nhận bằng ví của Admin

        // 2. Kiểm tra nếu người dùng đã có thẻ rồi
        if env.storage().instance().has(&DataKey::Pass(user.clone())) {
            panic!("User already has a TradePass");
        }

        // 3. Tạo thẻ mới
        let new_pass = TradePass {
            owner: user.clone(),
            is_active: true,
            level,
        };

        // 4. Lưu thẻ vào blockchain
        env.storage().instance().set(&DataKey::Pass(user), &new_pass);
    }