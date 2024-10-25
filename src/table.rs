pub fn render_table(rows: Vec<String>, cols: Vec<String>, matrix: Vec<Vec<char>>) {
    // Determine the width for the subject column
    let mut max_row_len = rows.iter().map(|r| r.len()).max().unwrap_or(7);

    if max_row_len < 7 {
        max_row_len = 7;
    }

    let col_width = max_row_len.max(max_row_len); // Ensures "Subject" fits
    let cell_width = 4; // Fixed width for language columns
    let cwm1 = cell_width - 1;
    let colm1 = col_width - 1;

    // Print the top border
    print!("┌{:─<col_width$}┬", "", col_width = max_row_len + 2);
    for (i, _) in cols.iter().enumerate() {
        if i == cols.len() - 1 {
            print!("{:─<cell_width$}┐", "─");
        } else {
            print!("{:─<cell_width$}┬", "─");
        }
    }
    println!();
    let x = col_width;

    // Print the header row (with "Subject" and language tags)
    print!("│{:<x$} │", " Subject", x = max_row_len + 1);
    for (i, lang) in cols.iter().enumerate() {
        if i == cols.len() - 1 {
            print!(" {:<cwm1$}│", lang);
        } else {
            print!(" {:<cwm1$}│", lang);
        }
    }
    println!();

    // Print the separator row
    print!("├{:─<col_width$}┼", "", col_width = max_row_len + 2);
    for (i, _) in cols.iter().enumerate() {
        if i == cols.len() - 1 {
            print!("{:─<cell_width$}┤", "─");
        } else {
            print!("{:─<cell_width$}┼", "─");
        }
    }
    println!();

    // Print each subject row with checkmarks/crosses
    for (i, subject) in rows.iter().enumerate() {
        print!("│ {:<col_width$} │", subject, col_width = col_width);
        for (j, &status) in matrix[i].iter().enumerate() {
            if j == cols.len() - 1 {
                print!(" {:<cwm1$}│", status);
            } else {
                print!(" {:<cwm1$}│", status);
            }
        }
        println!();
    }

    // Print the bottom border
    print!("└{:─<col_width$}┴", "", col_width = max_row_len + 2);
    for (i, _) in cols.iter().enumerate() {
        if i == cols.len() - 1 {
            print!("{:─<cell_width$}┘", "─");
        } else {
            print!("{:─<cell_width$}┴", "─");
        }
    }
    println!();
}

pub fn test_table() {
    let subjects = vec![
        "Bash".to_string(),
        "C++".to_string(),
        "Rust".to_string(),
        "Objeive-C".to_string(),
    ];
    let languages = vec![
        "en".to_string(),
        "de".to_string(),
        "fr".to_string(),
        "cz".to_string(),
    ];

    let matrix = vec![
        vec!['✔', '✘', '✔', '✔'],
        vec!['✘', '✔', '✔', '✔'],
        vec!['✔', '✔', '✘', '✔'],
        vec!['✔', '✔', '✘', '✔'],
        vec!['✔', '✔', '✘', '✔'],
    ];

    render_table(subjects, languages, matrix);
}
