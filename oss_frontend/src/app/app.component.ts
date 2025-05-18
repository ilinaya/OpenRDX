import { Component, OnInit } from '@angular/core';
import { TranslateService } from '@ngx-translate/core';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements OnInit {
  constructor(private translate: TranslateService) {}

  ngOnInit(): void {
    this.initializeLanguage();
  }

  private initializeLanguage(): void {
    const savedLang = localStorage.getItem('preferred_language');
    if (savedLang) {
      this.translate.use(savedLang);
    } else {
      const browserLang = navigator.language.split('-')[0];
      if (['en', 'es'].includes(browserLang)) {
        this.translate.use(browserLang);
        localStorage.setItem('preferred_language', browserLang);
      } else {
        this.translate.use('en');
        localStorage.setItem('preferred_language', 'en');
      }
    }
  }
}