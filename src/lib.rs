use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::Path,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;

/// 代表套件的依賴資訊
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}
/// 儲存套件的完整資訊
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub package_name: String,
    pub file_name: String,
    pub version: String,
    pub description: String,
    pub hash: String,
    pub dependencies: Option<Vec<Dependency>>,
}

impl PackageInfo {
    /// 建立一個新的 `PackageInfo` 實例
    ///
    /// # 參數
    /// - `package_name`: 套件名稱
    /// - `file_name`: 套件檔案名稱
    /// - `version`: 套件版本
    /// - `description`: 套件描述
    /// - `hash`: 套件檔案的雜湊值
    /// - `dependencies`: 可選的依賴列表
    ///
    /// # 回傳
    /// 回傳一個新的 `PackageInfo` 結構體
    pub fn new(
        package_name: String,
        file_name: String,
        version: String,
        description: String,
        hash: String,
        dependencies: Option<Vec<Dependency>>,
    ) -> PackageInfo {
        PackageInfo {
            package_name,
            file_name,
            version,
            description,
            hash,
            dependencies,
        }
    }
}
/// 用於處理 JSON 的存儲模組
pub struct JsonStorage<T> {
    _marker: std::marker::PhantomData<T>,
}
impl<T> JsonStorage<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    /// 從 JSON 檔案載入資料
    ///
    /// # 參數
    /// - `path`: JSON 檔案的路徑
    ///
    /// # 回傳
    /// 回傳載入的資料或錯誤
    pub fn from_json(path: &Path) -> io::Result<T> {
        let mut file_contents = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut file_contents)?;
        let data: T = serde_json::from_str(&file_contents)?;
        Ok(data)
    }

    /// 將資料存儲為 JSON 檔案
    ///
    /// # 參數
    /// - `data`: 要儲存的資料
    /// - `path`: 儲存檔案的路徑
    pub fn to_json(data: &T, path: &Path) -> io::Result<()> {
        let file = File::create(path)?;
        to_writer_pretty(file, &data)?;
        Ok(())
    }

    /// 從 URL 獲取並反序列化 JSON 資料（異步）
    ///
    /// # 參數
    /// - `url`: JSON 資料的 URL
    ///
    /// # 回傳
    /// 回傳載入的資料或錯誤
    pub async fn from_url(url: &str) -> Result<T, Box<dyn std::error::Error>> {
        let response = reqwest::get(url).await?.text().await?;
        let repo_info: T = serde_json::from_str(&response)?;
        Ok(repo_info)
    }
    /// 從字串反序列化 JSON 資料
    ///
    /// # 參數
    /// - `file_contents`: JSON 格式的字串
    ///
    /// # 回傳
    /// 回傳反序列化的資料或錯誤
    pub fn from_str_to(file_contents: &str) -> io::Result<T> {
        let data: T = serde_json::from_str(file_contents)?;
        Ok(data)
    }
}

/// 儲存庫的資訊管理模組
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RepoInfo {
    /// 儲存庫內的套件映射
    packages: HashMap<String, PackageBasicInfo>,
}
#[derive(Debug, Serialize, Deserialize)]
/// 套件的基本資訊
pub struct PackageBasicInfo {
    /// 套件的下載 URL
    pub url: String,
    /// 套件的檔案名稱
    pub file_name: String,
    /// 套件的版本
    pub version: String,
    /// 套件檔案的雜湊值
    pub hash: String,
    /// 套件的依賴列表（可選）
    pub dependencies: Option<Vec<Dependency>>,
}

impl RepoInfo {
    /// 建立一個新的 `RepoInfo` 實例
    pub fn new() -> Self {
        RepoInfo {
            packages: HashMap::new(),
        }
    }
    /// 檢查是否存在指定名稱的套件
    ///
    /// # 參數
    /// - `package_name`: 套件名稱
    ///
    /// # 回傳
    /// 若存在回傳 `true`，否則回傳 `false`
    pub fn has_package(&self, package_name: &str) -> bool {
        self.packages.contains_key(package_name)
    }
    /// 新增一個套件到儲存庫
    ///
    /// # 參數
    /// - `name`: 套件名稱
    /// - 其他參數：套件的相關資訊
    pub fn add_package(
        &mut self,
        name: String,
        url: String,
        file_name: String,
        version: String,
        hash: String,
        dependencies: Option<Vec<Dependency>>,
    ) {
        let package = PackageBasicInfo {
            url,
            file_name,
            version,
            hash,
            dependencies,
        };
        self.packages.insert(name, package);
    }
    /// 透過 `PackageBasicInfo` 新增一個套件
    pub fn add_package_with_info(&mut self, name: String, info: PackageBasicInfo) {
        self.packages.insert(name, info);
    }
    /// 根據名稱獲取套件
    ///
    /// # 參數
    /// - `package_name`: 套件名稱
    ///
    /// # 回傳
    /// 回傳套件資訊或錯誤
    pub fn get_package(&self, package_name: &str) -> Result<&PackageBasicInfo> {
        match self.packages.get(package_name) {
            Some(package) => Ok(package),
            None => Err(anyhow::anyhow!("Package '{}' not found.", package_name)),
        }
    }
    /// 根據名稱移除套件
    pub fn remove_package(&mut self, package_name: &str) -> Result<PackageBasicInfo> {
        match self.packages.remove(package_name) {
            Some(package) => Ok(package),
            None => Err(anyhow::anyhow!("Package '{}' not found.", package_name)),
        }
    }
    /// 更新儲存庫中的套件資訊
    pub fn update_package(
        &mut self,
        package_name: &str,
        url: Option<String>,
        file_name: Option<String>,
        version: Option<String>,
        hash: Option<String>,
        dependencies: Option<Vec<Dependency>>,
    ) {
        if let Some(existing_package) = self.packages.get_mut(package_name) {
            if let Some(new_url) = url {
                existing_package.url = new_url;
            }
            if let Some(new_file_name) = file_name {
                existing_package.file_name = new_file_name;
            }
            if let Some(new_version) = version {
                existing_package.version = new_version;
            }
            if let Some(new_hash) = hash {
                existing_package.hash = new_hash;
            }
            if let Some(new_dependencies) = dependencies {
                existing_package.dependencies = Some(new_dependencies);
            }
        } else {
            self.packages.insert(
                package_name.to_string(),
                PackageBasicInfo {
                    url: url.unwrap_or_default(),
                    file_name: file_name.unwrap_or_default(),
                    version: version.unwrap_or_default(),
                    hash: hash.unwrap_or_default(),
                    dependencies: None,
                },
            );
        }
    }
}