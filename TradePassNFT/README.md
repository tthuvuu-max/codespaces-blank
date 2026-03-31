# Title
TradePass NFT - Giải pháp cấp quyền giao dịch bảo mật

# Description
Người dùng mint NFT TradePass để nhận quyền tham gia giao dịch ngoài nhà nước (không chỉ là giấy thông hành) trong mọi hình thức giao dịch, đảm bảo an toàn và bảo mật dữ liệu cá nhân.

Mục đích dự án là giúp người dùng P2P, freelancer, nhà sáng tạo và người tham gia marketplace có thể tương tác giao dịch an toàn và minh bạch. 

Lý do thực hiện idea này là vì hiện nay các thủ tục xác minh truyền thống đòi hỏi người dùng phải chia sẻ CCCD hoặc giấy phép chính thức, dẫn tới nguy cơ lộ lọt dữ liệu nhạy cảm rất cao. Sử dụng giải pháp này giúp bảo vệ quyền riêng tư tuyệt đối cho người dùng.

# Tính năng 
Hợp đồng thông minh (Smart Contract) trên Soroban hiện tại xử lý các tính năng cốt lõi (MVP) sau:
* Khởi tạo hệ thống (init): Thiết lập địa chỉ ví của Admin đóng vai trò bên kiểm duyệt và cấp thẻ.
* Cấp thẻ (mint): Admin cấp NFT TradePass (không thể chuyển nhượng) đại diện cho quyền giao dịch gắn liền với địa chỉ ví của một người dùng cụ thể.
* Kiểm tra thẻ (check_pass): Cho phép bất kỳ ai cũng có thể tra cứu nhanh xem một địa chỉ ví có đang nắm giữ quyền giao dịch hợp lệ hay không mà không cần biết họ là ai.
* Thu hồi thẻ (revoke): Khóa hoặc vô hiệu hóa thẻ của người dùng vi phạm quy tắc để đảm bảo sự trong sạch cho nền tảng.

# Contract
Link tới contract: https://stellar.expert/explorer/testnet/contract/CAZSKCWI663ZMK2RW3XU4S4G6KWPL4YBR5KAY446VWAJW4BNPD6XNGSO 



# Future scopes
Tương lai dự án sẽ mở rộng thêm các tính năng tài chính phi tập trung (FinTech) sâu hơn như:
* Tích hợp thanh toán trực tiếp bằng XLM hoặc USDC cho các giao dịch P2P.
* Sử dụng đa chữ ký (multi-key), cơ chế khôi phục ví (recovery) và tính năng Clawback linh hoạt để xử lý tranh chấp hiệu quả.
* Ứng dụng sâu hơn các giải pháp bảo mật dữ liệu nâng cao như Zero-Knowledge Proof để không lưu vết thông tin người dùng lên on-chain.

# Profile
* Nickname/Tên: Thư Vũ
* Kỹ năng: Đang học tập và thực hành cơ bản về Smart Contract trên Soroban/Stellar, ngôn ngữ Rust ở mức nhập môn.