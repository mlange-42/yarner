#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#define OK 1                
#define usage_error 1       
#define cannot_open_file 2  
#define READ_ONLY 0
int status = OK;  
char *prog_name;  
long tot_word_count, tot_line_count, tot_char_count; 
void wc_print(char *which, long char_count, long word_count, long line_count)
{
    while (*which)
        switch (*which++) {
        case 'l': printf("%8ld", line_count);
            break;
        case 'w': printf("%8ld", word_count);
            break;
        case 'c': printf("%8ld", char_count);
            break;
        default:
            if ((status & 1) == 0) {
                fprintf(stderr, "\nUsage: %s [-lwc] [filename ...]\n", prog_name);
                status |= 1;
            }
        }
}
int main(int argc, char **argv)
{
    int file_count;  
    char *which;     
    int silent = 0;  
    int fd = 0;
    char buffer[BUFSIZ];     
    register char *ptr;      
    register char *buf_end;  
    register int c;          
    int in_word;             
    long word_count, line_count, char_count; 
    prog_name = argv[0];
    which = "lwc";   
    if (argc > 1 && *argv[1] == '-') {
        argv[1]++;
        if (*argv [1] == 's') silent = 1, argv [1]++;
        if (*argv [1]) which = argv [1];
        argc--;
        argv++;
    }
    file_count = argc - 1;
    argc--;
    do {
        if (file_count > 0 && (fd = open(*(++argv), READ_ONLY)) < 0) {
            fprintf(stderr, "%s: cannot open file %s\n", prog_name, *argv);
            status |= 2;
            file_count--;
            continue;
        }
        ptr = buf_end = buffer;
        line_count = word_count = char_count = 0;
        in_word = 0;
        while (1) {
            if (ptr >= buf_end) {
                ptr = buffer;
                c = read(fd, ptr, BUFSIZ);
                if (c <= 0) break;
                char_count += c;
                buf_end = buffer + c;
            }
            c = *ptr++;
            if (c > ' ' && c < 177) {    
                if (!in_word) {
                    word_count++;
                    in_word = 1;
                }
                continue;
            }
            if (c == '\n') line_count++;
            else if (c != ' ' && c != '\t') continue;
            in_word = 0;  
        }
        if (!silent) {
            wc_print(which, char_count, word_count, line_count);
            if (file_count) printf(" %s\n", *argv);  
            else printf("\n");                       
        }
        close(fd);
        tot_line_count += line_count;
        tot_word_count += word_count;
        tot_char_count += char_count;
    } while (--argc > 0);
    if (file_count > 1 || silent) {
        wc_print(which, tot_char_count, tot_word_count, tot_line_count);
        if (!file_count) printf("\n");
        else printf(" total in %d file%s\n", file_count, file_count > 1 ? "s" : "");
    }
    return status;
}
