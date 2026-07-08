#!/bin/bash
# Fix linux.rs manual_c_str_literals
sed -i 's/"_NET_ACTIVE_WINDOW\\0".as_ptr() as \*const i8/c"_NET_ACTIVE_WINDOW".as_ptr()/g' src/os/linux.rs
sed -i 's/"_NET_WM_NAME\\0".as_ptr() as \*const i8/c"_NET_WM_NAME".as_ptr()/g' src/os/linux.rs
sed -i 's/"_NET_WM_PID\\0".as_ptr() as \*const i8/c"_NET_WM_PID".as_ptr()/g' src/os/linux.rs

# Fix db.rs io_other_error
sed -i 's/std::io::Error::new(std::io::ErrorKind::Other, e.to_string())/std::io::Error::other(e.to_string())/g' src/storage/db.rs

# Fix ui/app.rs large_enum_variant
sed -i 's/UpdateAnalytics(crate::storage::AnalyticsData)/UpdateAnalytics(Box<crate::storage::AnalyticsData>)/g' src/ui/app.rs
sed -i 's/serde_json::to_string(&data).unwrap()/serde_json::to_string(data.as_ref()).unwrap()/g' src/ui/app.rs
